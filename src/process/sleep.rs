// use core::future::Future;
// use core::pin::Pin;
// use core::sync::atomic::Ordering;
// use core::task::{Context, Poll, Waker};
// use chrono::Duration;
// use crate::interrupts;
//
// type TimerId = u64;
//
// pub struct TimeoutWakeHandle {
//     timer: TimerId,
// }
//
// impl TimeoutWakeHandle {
//     fn set_waker(&self, waker: &Waker) {
//         unsafe {
//             EXECUTOR
//                 .as_ref()
//                 .unwrap()
//                 .timers
//                 .write()
//                 .get_mut(&self.timer)
//                 .unwrap()
//                 .1
//                 .replace(waker.clone());
//         }
//     }
// }
//
// impl Drop for TimeoutWakeHandle {
//     fn drop(&mut self) {
//         unsafe {
//             EXECUTOR
//                 .as_ref()
//                 .unwrap()
//                 .timers
//                 .write()
//                 .remove(&self.timer);
//         }
//     }
// }
//
pub fn tick_1ms() {
    // let inner = unsafe {
    //     if let Some(exec) = EXECUTOR.as_ref() {
    //         exec
    //     } else {
    //         return;
    //     }
    // };
    // inner.timers.write().retain(|_, arc| {
    //     let (time, waker) = arc.deref();
    //     if interrupts::now() < *time {
    //         true
    //     } else {
    //         if let Some(waker) = waker {
    //             waker.wake_by_ref();
    //         }
    //         false
    //     }
    // });
}
//
//
// fn wake_at_time(time: u64) -> TimeoutWakeHandle {
//     unsafe {
//         let id = EXECUTOR
//             .as_ref()
//             .unwrap()
//             .last_timer_id
//             .fetch_add(1, Ordering::SeqCst);
//         EXECUTOR
//             .as_ref()
//             .unwrap()
//             .timers
//             .write()
//             .insert(id, (time, None));
//         TimeoutWakeHandle { timer: id }
//     }
// }
//
//
// struct SleepFuture {
//     time: u64,
//     timeout_wake_handle: TimeoutWakeHandle,
// }
//
// impl Future for SleepFuture {
//     type Output = ();
//
//     fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
//         if interrupts::now() >= self.time {
//             Poll::Ready(())
//         } else {
//             self.timeout_wake_handle.set_waker(cx.waker());
//             Poll::Pending
//         }
//     }
// }
//
// pub fn sleep(timeout: Duration) -> impl Future<Output = ()> {
//     let time = interrupts::now() + timeout.num_milliseconds() as u64;
//     let handle = wake_at_time(time);
//     SleepFuture {
//         time,
//         timeout_wake_handle: handle,
//     }
// }