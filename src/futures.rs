use crate::interrupts;
use alloc::boxed::Box;
use alloc::collections::VecDeque;
use alloc::sync::Arc;
use alloc::task::Wake;
use chrono::Duration;
use core::cell::RefCell;
use core::future::Future;
use core::ops::{Deref, DerefMut};
use core::option::Option::Some;
use core::pin::Pin;
use core::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use core::task::{Context, Poll, Waker};
use crossbeam_queue::SegQueue;
use hashbrown::HashMap;
use spin::RwLock;

static WAKE_CALLED: AtomicBool = AtomicBool::new(false);
static mut EXECUTOR: Option<ExecutorInner> = None;

type TimerId = u64;

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

#[deprecated]
pub struct TimeoutWakeHandle {
    timer: TimerId,
}

impl TimeoutWakeHandle {
    fn set_waker(&self, waker: &Waker) {
        unsafe {
            EXECUTOR
                .as_ref()
                .unwrap()
                .timers
                .write()
                .get_mut(&self.timer)
                .unwrap()
                .1
                .replace(waker.clone());
        }
    }
}

impl Drop for TimeoutWakeHandle {
    fn drop(&mut self) {
        unsafe {
            EXECUTOR
                .as_ref()
                .unwrap()
                .timers
                .write()
                .remove(&self.timer);
        }
    }
}

struct ExecutorInner {
    queue: RwLock<VecDeque<Task>>,
    add_queue: SegQueue<Task>,
    timers: RwLock<HashMap<TimerId, (u64, Option<Waker>)>>,
    last_timer_id: AtomicU64,
}

#[deprecated]
pub fn init() {
    unsafe {
        EXECUTOR = Some(ExecutorInner {
            queue: RwLock::new(VecDeque::new()),
            add_queue: SegQueue::new(),
            timers: RwLock::new(HashMap::new()),
            last_timer_id: AtomicU64::new(0),
        })
    }
}

#[deprecated]
pub fn tick_1ms() {
    let inner = unsafe {
        if let Some(exec) = EXECUTOR.as_ref() {
            exec
        } else {
            return;
        }
    };
    inner.timers.write().retain(|_, arc| {
        let (time, waker) = arc.deref();
        if interrupts::now() < *time {
            true
        } else {
            if let Some(waker) = waker {
                waker.wake_by_ref();
            }
            false
        }
    });
}


#[deprecated]
fn wake_at_time(time: u64) -> TimeoutWakeHandle {
    unsafe {
        let id = EXECUTOR
            .as_ref()
            .unwrap()
            .last_timer_id
            .fetch_add(1, Ordering::SeqCst);
        EXECUTOR
            .as_ref()
            .unwrap()
            .timers
            .write()
            .insert(id, (time, None));
        TimeoutWakeHandle { timer: id }
    }
}


#[deprecated]
pub fn spawn<F>(future: F)
where
    F: Future<Output = ()> + 'static,
{
    crate::process::spawn_kernel(future);
    // unsafe {
    //     if let Some(executor) = EXECUTOR.as_ref() {
    //         executor.add_queue.push(Task {
    //             future: Box::pin(future),
    //         });
    //     } else {
    //         // kblog!("Futures", "Spawn invoked before executor started") FIXME can deadlock
    //     }
    // }
}

struct SleepFuture {
    time: u64,
    timeout_wake_handle: TimeoutWakeHandle,
}

impl Future for SleepFuture {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if interrupts::now() >= self.time {
            Poll::Ready(())
        } else {
            self.timeout_wake_handle.set_waker(cx.waker());
            Poll::Pending
        }
    }
}

#[deprecated]
pub fn sleep(timeout: Duration) -> impl Future<Output = ()> {
    let time = interrupts::now() + timeout.num_milliseconds() as u64;
    let handle = wake_at_time(time);
    SleepFuture {
        time,
        timeout_wake_handle: handle,
    }
}

/// Schedules future on main kernel loop
macro_rules! spawn {
    ($arg:expr) => {
        crate::futures::spawn($arg)
    };
}
