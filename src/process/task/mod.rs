use alloc::boxed::Box;
use alloc::string::String;
use core::future::Future;
use core::pin::Pin;

use runtime::ProcessRuntime;

mod ctx;
mod int;
mod runtime;
use crate::process::task::ctx::TaskContext;
use crate::process::task::runtime::ProcessRuntimeHandle;
pub use int::run;

pub enum ProcessTaskState {
    Ready(Pin<Box<dyn Future<Output = ()>>>),
    Paused(TaskContext),
}

#[derive(Clone)]
pub struct ProcessTaskInfo {
    pub name: String,
}

pub struct ProcessTask {
    info: ProcessTaskInfo,
    state: ProcessTaskState,
}

static mut HANDLE: Option<ProcessRuntimeHandle> = None;

pub fn setup() {
    unsafe {
        let runtime = ProcessRuntime::new();
        HANDLE = Some(runtime.handle());
        int::setup(runtime);
    }
}

pub fn add_task<F>(info: ProcessTaskInfo, future: F)
where
    F: Future<Output = ()> + 'static,
{
    unsafe {
        HANDLE
            .as_mut()
            .expect("Process runtime not started")
            .add(ProcessTask {
                info,
                state: ProcessTaskState::Ready(Box::pin(future)),
            })
    }
}
