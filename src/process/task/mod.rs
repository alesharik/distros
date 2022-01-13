use libkernel::task::{Task, TASK_QUEUE_MEM};
use core::pin::Pin;
use core::future::Future;
use crate::memory;
use x86_64::VirtAddr;
use x86_64::structures::paging::PageTableFlags;
use crate::memory::util::{MemoryToken, MemoryError};

mod runtime;
pub use runtime::ProcessRuntime;

pub struct ProcessTask {
    task: Task,
}

impl ProcessTask {
    pub fn try_take() -> Option<ProcessTask> {
        unsafe { Task::take_from_queue() }.map(|t| ProcessTask {
            task: t,
        })
    }

    pub fn future(&mut self) -> Pin<&mut dyn Future<Output=u8>> {
        self.task.future.as_mut()
    }

    pub fn is_primary(&self) -> bool {
        self.task.is_primary
    }
}

pub unsafe fn setup_current_context() -> Result<MemoryToken, MemoryError> {
    memory::util::static_map_memory(
        VirtAddr::new_truncate(TASK_QUEUE_MEM),
        4096,
        PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::USER_ACCESSIBLE,
    )
}