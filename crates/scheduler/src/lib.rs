#![no_std]
#![feature(abi_x86_interrupt)]
#![feature(naked_functions)]

extern crate alloc;

mod nice;
mod registry;
mod scheduler;

use alloc::boxed::Box;
use alloc::string::String;
use alloc::sync::Arc;
pub use nice::NiceLevel;
pub use registry::TaskBuilder;
pub use scheduler::start as sched_start;
use spin::{Mutex, RwLock};

use crate::registry::TaskRegistry;
use crate::scheduler::TaskState;

static mut REGISTRY: Option<RwLock<TaskRegistry>> = None;

#[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
#[repr(transparent)]
pub struct TaskId(u64);

impl TaskId {
    const EMPTY: TaskId = TaskId(0);
}

bitflags::bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct TaskFlags: u32 {
        const NOPREEMPT = 0b00000001;
    }
}

pub fn init() {
    unsafe {
        REGISTRY = Some(RwLock::new(TaskRegistry::new()));
    }
    scheduler::init();
}

pub fn spawn(task: TaskBuilder) -> TaskId {
    let mut registry = unsafe {
        REGISTRY
            .as_mut()
            .expect("Task registry not initialized")
            .write()
    };
    let task_id: Arc<Mutex<TaskId>> = Arc::new(Mutex::new(TaskId::EMPTY));
    let task_id_1 = task_id.clone();
    let tid = registry.spawn(task.wrap_executable(|future| {
        Box::pin(async move {
            future.await;
            unsafe {
                if let Some(reg) = REGISTRY.as_ref() {
                    let mut reg = reg.write();
                    let tid = task_id_1.lock();
                    reg.remove(*tid);
                }
            }
        })
    }));
    *task_id.lock() = tid;
    tid
}

pub fn get_name(task: TaskId) -> Option<Option<String>> {
    let registry = unsafe {
        REGISTRY
            .as_mut()
            .expect("Task registry not initialized")
            .read()
    };
    registry.get_name(task)
}

pub fn get_nice(task: TaskId) -> Option<NiceLevel> {
    let registry = unsafe {
        REGISTRY
            .as_mut()
            .expect("Task registry not initialized")
            .read()
    };
    registry.get_nice(task)
}

pub fn get_state(task: TaskId) -> Option<TaskState> {
    scheduler::get_state(task)
}

pub fn current_task() -> TaskId {
    scheduler::current().expect("Should not be executed in empty task")
}

pub fn set_name(task: TaskId, name: impl Into<String>) {
    let mut registry = unsafe {
        REGISTRY
            .as_mut()
            .expect("Task registry not initialized")
            .write()
    };
    registry.set_name(task, name);
}
