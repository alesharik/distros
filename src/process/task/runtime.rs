//! Provides runtime to run tasks
use alloc::borrow::ToOwned;
use crate::interrupts;
use crate::memory::util::{MemoryError};
use crate::process::task::ctx::{Regs, TaskContext};
use crate::process::task::{ProcessTask, ProcessTaskInfo, ProcessTaskState};
use alloc::boxed::Box;
use alloc::sync::Arc;
use alloc::task::Wake;
use core::cell::RefCell;
use core::ops::{Deref, DerefMut};

use core::sync::atomic::{AtomicBool, Ordering};
use core::task::{Context, Poll};
use crossbeam_queue::SegQueue;
use x86_64::structures::idt::InterruptStackFrame;


const QUEUE_TASK_CAP: i32 = 1000;

struct SendWrapper<T> {
    data: *mut T,
}

impl<T> SendWrapper<T> {
    pub unsafe fn new(data: T) -> SendWrapper<T> {
        SendWrapper {
            data: Box::into_raw(Box::new(data)),
        }
    }
}

unsafe impl<T> Send for SendWrapper<T> {}
unsafe impl<T> Sync for SendWrapper<T> {}

impl<T> Deref for SendWrapper<T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { &*self.data }
    }
}

impl<T> DerefMut for SendWrapper<T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.data }
    }
}

impl<T> Drop for SendWrapper<T> {
    fn drop(&mut self) {
        unsafe {
            Box::from_raw(self.data);
        }
    }
}

struct TaskWaker {
    task: SendWrapper<RefCell<Option<ProcessTask>>>,
    queue: SendWrapper<Arc<SegQueue<ProcessTask>>>,
    wake_called: Arc<AtomicBool>,
}

impl TaskWaker {
    unsafe fn new(queue: Arc<SegQueue<ProcessTask>>) -> (Arc<TaskWaker>, Arc<AtomicBool>) {
        let wake_called = Arc::new(AtomicBool::new(false));
        (
            Arc::new(TaskWaker {
                task: SendWrapper::new(RefCell::new(None)),
                queue: SendWrapper::new(queue),
                wake_called: wake_called.clone(),
            }),
            wake_called,
        )
    }
}

impl Wake for TaskWaker {
    fn wake(self: Arc<Self>) {
        if let Some(task) = (*self.task).borrow_mut().take() {
            self.queue.push(task)
        } else {
            self.wake_called.store(true, Ordering::SeqCst);
        }
    }
}

pub struct ProcessRuntimeHandle {
    queue: Arc<SegQueue<ProcessTask>>,
}

impl ProcessRuntimeHandle {
    /// Add new task to runtime
    pub fn add(&self, task: ProcessTask) {
        self.queue.push(task)
    }
}

pub struct ProcessRuntime {
    queue: Arc<SegQueue<ProcessTask>>,
    current_task_info: Option<ProcessTaskInfo>,
    task_running: AtomicBool
}

impl ProcessRuntime {
    /// Create new runtime and allocate task queue in current memory space
    pub unsafe fn new() -> Result<Self, MemoryError> {
        let queue = SegQueue::new();
        queue.push(ProcessTask {
            info: ProcessTaskInfo { name: "empty".to_owned() },
            state: ProcessTaskState::Ready(Box::pin(futures::future::pending()))
        });
        Ok(ProcessRuntime {
            queue: Arc::new(queue),
            current_task_info: None,
            task_running: AtomicBool::new(true) // to start main cycle
        })
    }

    pub fn handle(&self) -> ProcessRuntimeHandle {
        return ProcessRuntimeHandle {
            queue: self.queue.clone(),
        };
    }

    unsafe fn save_current_task(&mut self, stack_frame: &InterruptStackFrame, regs: &Regs) {
        if let Some(mut task) = self.current_task_info.take() {
            self.queue.push(ProcessTask {
                info: task,
                state: ProcessTaskState::Paused(TaskContext::fill_from(&stack_frame, regs))
            });
        }
    }

    pub unsafe fn int(&mut self, mut stack_frame: InterruptStackFrame, regs: &mut Regs) {
        if !self.task_running.load(Ordering::SeqCst) {
            interrupts::eoi();
            return; // ok, we continue main loop
        }
        let task = self.queue.pop();
        if let None = task {
            interrupts::eoi();
            return; // nothing to schedule, returning to main loop
        }
        self.save_current_task(&stack_frame, regs);
        let task = task.unwrap();

        let last = self.current_task_info.clone();
        self.current_task_info = Some(task.info.clone());
        let task_info = task.info.clone();
        match task.state {
            ProcessTaskState::Ready(mut future) => {
                let (task_waker, wake_called) = TaskWaker::new(self.queue.clone());
                let waker_cloned = &task_waker.clone().into();
                let mut context = Context::from_waker(waker_cloned);
                interrupts::eoi();
                let result = future.as_mut().poll(&mut context);
                interrupts::no_int(|| {
                    self.current_task_info = last; // task finished execution, so we remove it from current
                    match result {
                        Poll::Ready(()) => {
                            // good! Forgetting task
                        }
                        Poll::Pending => {
                            let new_task = ProcessTask {
                                info: task_info,
                                state: ProcessTaskState::Ready(future),
                            };
                            if wake_called.load(Ordering::SeqCst) {
                                // setting up waker
                                self.queue.push(new_task);
                            } else {
                                (*task_waker.task).borrow_mut().replace(new_task);
                            }
                        }
                    }
                });
                // so, task is handled, give control to main loop
            }
            ProcessTaskState::Paused(context) => {
                context.save_info(stack_frame.as_mut().extract_inner(), regs);
                // continue paused task execution
            }
        }
        interrupts::eoi();
    }

    pub unsafe fn run(&mut self) -> ! {
        loop {
            let task = self.queue.pop();
            if let None = task {
                x86_64::instructions::hlt();
                continue;
            }
            let mut task = task.unwrap();

            // interrupts::disable_interrupts();

            self.current_task_info = Some(task.info.clone()); // selected task to run
            let task_info = task.info.clone();
            match task.state {
                ProcessTaskState::Ready(mut future) => {
                    let (task_waker, wake_called) = TaskWaker::new(self.queue.clone());
                    let waker_cloned = &task_waker.clone().into();
                    let mut context = Context::from_waker(waker_cloned);
                    self.task_running.store(true, Ordering::SeqCst);
                    let result = future.as_mut().poll(&mut context);
                    self.task_running.store(false, Ordering::SeqCst);
                    interrupts::no_int(|| {
                        self.current_task_info = None; // task finished execution, so we remove it from current
                        match result {
                            Poll::Ready(()) => {
                                // good! Forgetting task
                            }
                            Poll::Pending => {
                                let new_task = ProcessTask {
                                    info: task_info,
                                    state: ProcessTaskState::Ready(future),
                                };
                                if wake_called.load(Ordering::SeqCst) {
                                    // setting up waker
                                    self.queue.push(new_task);
                                } else {
                                    (*task_waker.task).borrow_mut().replace(new_task);
                                }
                            }
                        }
                    });
                    // so, task is handled and we want next task
                }
                ProcessTaskState::Paused(context) => {
                    // can't handle, give it to int
                    self.queue.push(ProcessTask {
                        info: task_info,
                        state: ProcessTaskState::Paused(context)
                    });
                }
            }
        }
    }
}
