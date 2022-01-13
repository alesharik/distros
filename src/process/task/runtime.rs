//! Provides runtime to run tasks
use crate::interrupts;
use alloc::boxed::Box;
use alloc::sync::Arc;
use alloc::task::Wake;
use core::cell::RefCell;
use core::ops::{Deref, DerefMut};
use core::pin::Pin;
use core::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering};
use core::task::{Context, Poll};
use crossbeam_queue::SegQueue;
use crate::process::task::{ProcessTask, setup_current_context};
use crate::memory::util::{MemoryError, MemoryToken};

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
    queue: Arc<SegQueue<ProcessTask>>,
    wake_called: Arc<AtomicBool>,
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

pub struct ProcessRuntime {
    add_queue: SegQueue<ProcessTask>,
    queue: Arc<SegQueue<ProcessTask>>,
    wake_called: Arc<AtomicBool>,
    task_queue_memory_token: MemoryToken,
    active_primary_task_count: AtomicU32,
}

impl ProcessRuntime {
    /// Create new runtime and allocate task queue in current memory space
    pub unsafe fn new() -> Result<Self, MemoryError> {
        Ok(ProcessRuntime {
            add_queue: SegQueue::new(),
            queue: Arc::new(SegQueue::new()),
            wake_called: Arc::new(AtomicBool::new(false)),
            task_queue_memory_token: setup_current_context()?,
            active_primary_task_count: AtomicU32::new(0)
        })
    }

    /// Add new task to runtime
    pub fn add(&self, task: ProcessTask) {
        self.add_queue.push(task)
    }

    /// Run loop
    ///
    /// # Returns
    /// Process exit code
    pub fn run(&self) -> u8 {
        loop {
            while let Some(task) = self.add_queue.pop() {
                if task.is_primary() {
                    self.active_primary_task_count.fetch_add(1, Ordering::SeqCst);
                }
                self.queue.push(task)
            }
            while let Some(task) = ProcessTask::try_take() {
                if task.is_primary() {
                    self.active_primary_task_count.fetch_add(1, Ordering::SeqCst);
                }
                self.queue.push(task)
            }
            let task = self.queue.pop();
            if let Some(mut task) = task {
                let task_waker = unsafe {
                    Arc::new(TaskWaker {
                        task: SendWrapper::new(RefCell::new(None)),
                        queue: self.queue.clone(),
                        wake_called: self.wake_called.clone(),
                    })
                };
                self.wake_called.store(false, Ordering::SeqCst);
                match task.future()
                    .poll(&mut Context::from_waker(&task_waker.clone().into()))
                {
                    Poll::Ready(result) => {
                        if task.is_primary() {
                            if result != 0 {
                                break result
                            } else if self.active_primary_task_count.fetch_sub(1, Ordering::SeqCst) <= 1 {
                                break 0
                            }
                        } else {
                            // todo print background task result if != 0
                        }
                    },
                    Poll::Pending => {
                        if self.wake_called.load(Ordering::SeqCst) {
                            self.queue.push(task);
                        } else {
                            (*task_waker.task).borrow_mut().replace(task);
                        }
                    }
                }
            } else {
                // todo make process sleep / yield
            }
        }
    }
}