use alloc::borrow::ToOwned;
use alloc::vec::Vec;
use core::future::Future;

mod task;
pub mod sleep;

pub use task::{setup, run};
use crate::process::task::ProcessTaskInfo;

struct Thread {
}

struct Process {
    threads: Vec<Thread>,
}

pub fn spawn_kernel<F>(name: &str, future: F)
    where
        F: Future<Output = ()> + 'static,
{
    task::add_task(ProcessTaskInfo {
        name: name.to_owned()
    }, future);
}

/// Schedules future on main kernel loop
macro_rules! spawn {
    ($name:expr => $arg:expr) => {
        crate::process::spawn_kernel($name, $arg)
    };
}