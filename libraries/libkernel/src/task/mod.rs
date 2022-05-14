//! Process tasking module

use alloc::boxed::Box;
use alloc::string::String;
use core::future::Future;

use core::pin::Pin;

/// This structure represents process task (also called `thread` in popular OSes). Task is a future that
/// schedules with task scheduler. When the scheduler runs the task, it invokes `Future::poll` method and waits for result.
///
/// When a task returns `Poll::Ready`, scheduler checks return code, throws error if necessary, and forgets about task.
///
/// When a task returns `Poll::Pending`, it will be put in a scheduler queue and will be scheduled at some time in future.
pub struct Task {
    /// Human readable task name
    pub name: String,

    /// Task future
    pub future: Pin<Box<dyn Future<Output = ()>>>,
}

unsafe impl Send for Task {}
