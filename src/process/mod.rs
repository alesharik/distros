use alloc::borrow::ToOwned;
use crate::memory::Liballoc;
use alloc::vec::Vec;
use core::future::Future;

mod task;

pub use task::{setup, run};
use crate::process::task::ProcessTaskInfo;

struct Thread {
}

struct Process {
    liballoc: Liballoc,
    threads: Vec<Thread>,
}

pub fn spawn_kernel<F>(future: F)
    where
        F: Future<Output = ()> + 'static,
{
    task::add_task(ProcessTaskInfo {
        name: "ktask".to_owned()
    }, future);
}