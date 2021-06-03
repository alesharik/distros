use alloc::boxed::Box;
use core::future::Future;
use core::cell::RefCell;
use core::pin::Pin;
use alloc::sync::Arc;
use alloc::task::Wake;
use alloc::collections::VecDeque;
use alloc::rc::Rc;
use core::task::Context;
use spin::RwLock;
use futures::SinkExt;
use core::borrow::{BorrowMut, Borrow};
use core::sync::atomic::{AtomicBool, Ordering};
use crossbeam_queue::SegQueue;
use futures::task::{Spawn, SpawnError};
use futures::future::FutureObj;
use core::option::Option::Some;
use core::ops::{Deref, DerefMut};

static WAKE_CALLED: AtomicBool = AtomicBool::new(false);
static mut EXECUTOR: Option<ExecutorInner> = None;

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

struct Task {
    future: Pin<Box<dyn Future<Output = ()>>>,
}

struct TaskWaker {
    task: SendWrapper<RefCell<Option<Task>>>,
}

impl Wake for TaskWaker {
    fn wake(self: Arc<Self>) {
        if let Some(task) = (*self.task).borrow_mut().take() {
            unsafe {
                EXECUTOR
                    .as_ref()
                    .expect("Executor not started")
                    .queue
                    .write()
                    .push_back(task);
            }
        } else {
            WAKE_CALLED.store(true, Ordering::SeqCst);
        }
    }
}

struct ExecutorInner {
    queue: RwLock<VecDeque<Task>>,
    add_queue: SegQueue<Task>,
}

pub fn init() {
    unsafe {
        EXECUTOR = Some(ExecutorInner {
            queue: RwLock::new(VecDeque::new()),
            add_queue: SegQueue::new(),
        })
    }
}

pub fn run() -> ! {
    kblog!("Futures", "Runtime running");
    loop {
        unsafe {
            while let Some(task) = EXECUTOR.as_ref().unwrap().add_queue.pop() {
                EXECUTOR.as_ref().unwrap().queue.write().push_back(task);
            }
        }
        let task = unsafe {
            let task = EXECUTOR.as_ref().unwrap().queue.write().pop_front();
            task
        };
        if let Some(mut task) = task {
            let mut task_waker = unsafe {
                Arc::new(TaskWaker {
                    task: SendWrapper::new(RefCell::new(None)),
                })
            };
            WAKE_CALLED.store(false, Ordering::SeqCst);
            if task
                .future
                .as_mut()
                .poll(&mut Context::from_waker(&task_waker.clone().into()))
                .is_pending()
            {
                if WAKE_CALLED.load(Ordering::SeqCst) {
                    unsafe {
                        EXECUTOR.as_ref().unwrap().queue.write().push_back(task);
                    }
                } else {
                    (*task_waker.task).borrow_mut().replace(task);
                }
            }
        } else {
            x86_64::instructions::hlt(); // FIXME can have interrupt right before this instruction
        }
    }
}

pub fn spawn<F>(future: F)
    where
        F: Future<Output = ()> + Sync + Send + 'static,
{
    unsafe {
        EXECUTOR.as_ref().expect("Executor not started").add_queue.push(Task {
            future: Box::pin(future)
        });
    }
}