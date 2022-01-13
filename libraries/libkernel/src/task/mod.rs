//! Process tasking module

use alloc::string::String;
use core::pin::Pin;
use core::future::Future;
use alloc::boxed::Box;
use core::mem::size_of;

/// Memory block (4KiB) where task queue is stored. All tasks from this queue will be taken on next
/// task switch and added as running tasks to task scheduler.
pub const TASK_QUEUE_MEM: u64 = 0x_5444_4446_0000;

/// This structure represents process task (also called `thread` in popular OSes). Task is a future that
/// schedules with task scheduler. When the scheduler runs the task, it invokes `Future::poll` method and waits for result.
///
/// When a task returns `Poll::Ready`, scheduler checks return code, throws error if necessary, and forgets about task.
///
/// When a task returns `Poll::Pending`, it will be put in a scheduler queue and will be scheduled at some time in future.
///
/// Return code `0` means task finished successfully, all other codes means that task failed to complete and current process
/// will be shut down with error code.
///
/// If all `primary` tasks completes, then process will be considered `complete` and all other tasks will be forgotten.
pub struct Task {
    /// Human readable task name
    pub name: String,

    /// Task future
    pub future: Pin<Box<dyn Future<Output = u8>>>,

    /// `false` means that task is a `daemon` and can be shut down when all `primary` tasks are complete
    pub is_primary: bool,
}

impl Task {
    /// Put this task in queue
    ///
    /// # Returns
    /// `true` - operation successful, `false` - queue is full
    ///
    /// # Panics
    /// Will panic when task queue was concurrently modified while this method is executed
    #[must_use]
    pub unsafe fn put_in_queue(self) -> bool {
        let count = (TASK_QUEUE_MEM as *mut u32).read_volatile();
        if count >= ((4096 - 4) / (size_of::<Task>() as u32)) as u32 {
            false
        } else {
            if (TASK_QUEUE_MEM as *mut u32).read_volatile() != count {
                panic!("Concurrent task queue modification is not allowed")
            }
            (TASK_QUEUE_MEM as *mut u32).write_volatile(count + 1);
            ((TASK_QUEUE_MEM + 4 + (count as u64) * (size_of::<Task>() as u64)) as *mut Task).write_volatile(self);
            true
        }
    }

    /// Take task from queue
    ///
    /// # Returns
    /// Will return task if have any in queue, otherwise return `None`
    ///
    /// # Panics
    /// Will panic when task queue was concurrently modified while this method is executed
    pub unsafe fn take_from_queue() -> Option<Task> {
        let count = (TASK_QUEUE_MEM as *mut u32).read_volatile();
        if count > 0 {
            if (TASK_QUEUE_MEM as *mut u32).read_volatile() != count {
                panic!("Concurrent task queue modification is not allowed")
            }
            (TASK_QUEUE_MEM as *mut u32).write_volatile(count - 1);
            let task = ((TASK_QUEUE_MEM + 4 + (count as u64) * (size_of::<Task>() as u64)) as *mut Task).read_volatile();
            Some(task)
        } else {
            None
        }
    }
}