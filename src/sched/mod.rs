use crate::sched::ctx::TaskContext;
use core::sync::atomic::AtomicPtr;
use core::cmp::Ordering;
use crate::sched::scheduler::CpuTaskScheduler;
use crate::interrupts::INT_LAPIC_TIMER;

mod ctx;
mod int;
mod scheduler;

#[derive(Clone, Eq, PartialEq)]
pub enum TaskState {
    Running,
    Ready,
}

pub struct Task {
    pub context: TaskContext,
    pub state: TaskState,
}

impl Task {
    pub const fn new() -> Task {
        Task {
            context: TaskContext::new(),
            state: TaskState::Ready,
        }
    }
}

static mut SCHEDULER: CpuTaskScheduler = CpuTaskScheduler::new();

pub fn start() {
    crate::interrupts::set_handler(INT_LAPIC_TIMER, int::switch_context);
    crate::interrupts::start_lapic_timer();
}