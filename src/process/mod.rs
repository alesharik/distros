use alloc::vec::Vec;
use crate::memory::Liballoc;
use crate::process::task::ProcessRuntime;

mod task;

struct Thread {
    runtime: ProcessRuntime
}

struct Process {
    liballoc: Liballoc,
    threads: Vec<Thread>
}