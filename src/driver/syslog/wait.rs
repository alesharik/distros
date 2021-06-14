//! Future that wakes on syslog messaeg

use futures::Future;
use core::task::{Context, Poll};
use core::pin::Pin;
use spin::{Lazy, RwLock, Mutex};
use alloc::vec::Vec;
use crossbeam_queue::SegQueue;
use futures::task::Waker;
use alloc::sync::{Weak, Arc};
use core::sync::atomic::{AtomicBool, Ordering};

struct SyslogWaitHandle {
    waker: Waker,
    was_waked: AtomicBool,
}

impl SyslogWaitHandle {
    fn new(waker: Waker) -> Self {
        SyslogWaitHandle {
            waker,
            was_waked: AtomicBool::new(false),
        }
    }

    fn wake(&self) {
        if self.was_waked.compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst).is_ok() {
            self.waker.wake_by_ref();
        }
    }
}

static HANDLES: Lazy<SegQueue<Weak<SyslogWaitHandle>>> = Lazy::new(|| SegQueue::new());

struct SyslogWaitFuture {
    handle: Option<Arc<SyslogWaitHandle>>,
}

impl Future for SyslogWaitFuture {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let me = self.get_mut();
        if let Some(handle) = me.handle.take() {
            if handle.was_waked.load(Ordering::SeqCst) {
                return Poll::Ready(());
            }
        }
        let handle = Arc::new(SyslogWaitHandle::new(cx.waker().clone()));
        me.handle = Some(handle.clone());
        HANDLES.push(Arc::downgrade(&handle));
        Poll::Pending
    }
}

pub fn wakeup() {
    while let Some(handle) = HANDLES.pop() {
        if let Some(handle) = handle.upgrade() {
            handle.wake();
        }
    }
}

pub fn wait_for_syslog() -> impl Future<Output = ()> {
    SyslogWaitFuture { handle: None }
}