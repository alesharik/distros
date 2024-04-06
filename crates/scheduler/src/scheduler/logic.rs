use crate::scheduler::context::{Regs, TaskContext};
use crate::scheduler::TaskState;
use crate::{NiceLevel, TaskFlags, TaskId};
use alloc::boxed::Box;
use alloc::rc::Rc;
use alloc::string::ToString;
use alloc::sync::Arc;
use alloc::task::Wake;
use core::future::Future;
use core::pin::Pin;
use core::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use core::task::{Context, Poll};
use core::time::Duration;
use distros_memory_stack::{
    find_buffer, new_buffer, StackBuffer, StackBufferHandle, KERNEL_STACK_SIZE,
};
use distros_timer_tsc::tsc;
use hashbrown::HashMap;
use intrusive_collections::{intrusive_adapter, KeyAdapter, RBTree, RBTreeLink};
use log::warn;
use spin::Mutex;
use x2apic::lapic::TimerDivide;
use x86_64::instructions::interrupts::without_interrupts;
use x86_64::structures::idt::InterruptStackFrame;
use x86_64::VirtAddr;

const DEADLINE: Duration = Duration::from_millis(1000);
const NICE_PERIOD: Duration = Duration::from_nanos(500 * 1000);

enum WaitingTaskState {
    Paused(TaskContext),
    Ready(Pin<Box<dyn Future<Output = ()>>>),
}

unsafe impl Send for WaitingTaskState {}
unsafe impl Sync for WaitingTaskState {}

struct WaitingTask {
    link: RBTreeLink,
    id: TaskId,
    state: WaitingTaskState,
    run_time: u64,
    nice: NiceLevel,
    flags: TaskFlags,
    buffer_handle: Option<StackBufferHandle>,
}

struct RunningTask {
    id: TaskId,
    nice: NiceLevel,
    flags: TaskFlags,
    stack_handle: Option<StackBufferHandle>,
}

intrusive_adapter!(WaitingTaskAdapter = Box<WaitingTask>: WaitingTask { link: RBTreeLink });
impl<'a> KeyAdapter<'a> for WaitingTaskAdapter {
    type Key = u64;
    fn get_key(&self, x: &'a WaitingTask) -> u64 {
        x.run_time
    }
}

#[derive(Clone)]
struct TaskStates {
    task_states: Arc<Mutex<HashMap<TaskId, TaskState>>>,
}

unsafe impl Sync for TaskStates {}
unsafe impl Send for TaskStates {}

impl TaskStates {
    fn new() -> Self {
        TaskStates {
            task_states: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    fn get_state(&self, task: TaskId) -> Option<TaskState> {
        without_interrupts(|| {
            let states = self.task_states.lock();
            states.get(&task).copied()
        })
    }

    fn set_state(&self, task: TaskId, state: TaskState) {
        let mut states = self.task_states.lock();
        states.insert(task, state);
    }

    fn remove_state(&self, task: TaskId) {
        let mut states = self.task_states.lock();
        states.remove(&task);
    }
}

struct TaskWaker {
    task: Mutex<Option<WaitingTask>>,
    wake_called: AtomicBool,
    queue: Arc<Mutex<RBTree<WaitingTaskAdapter>>>,
}

impl TaskWaker {
    pub fn new(queue: &Arc<Mutex<RBTree<WaitingTaskAdapter>>>) -> Self {
        TaskWaker {
            task: Mutex::new(None),
            wake_called: AtomicBool::new(false),
            queue: queue.clone(),
        }
    }
}

impl Wake for TaskWaker {
    fn wake(self: Arc<Self>) {
        let mut task = self.task.lock();
        if let Some(task) = task.take() {
            let mut queue = self.queue.lock();
            queue.insert(Box::new(task));
        } else {
            self.wake_called.store(true, Ordering::SeqCst);
        }
    }
}

unsafe impl Send for TaskWaker {}

unsafe impl Sync for TaskWaker {}

pub struct Scheduler {
    task_states: TaskStates,
    waiting_tasks: Arc<Mutex<RBTree<WaitingTaskAdapter>>>,
    current_task: Option<RunningTask>,
    id_counter: AtomicU64,
    tsc_deadline: bool,
    lapic_freq: u64,
}

impl Scheduler {
    pub const DIVIDER: u64 = 32;
    pub const LAPIC_DIVIDER: TimerDivide = TimerDivide::Div32;

    pub fn new(tsc_deadline: bool) -> Scheduler {
        Scheduler {
            task_states: TaskStates::new(),
            waiting_tasks: Arc::new(Mutex::new(RBTree::new(WaitingTaskAdapter::new()))),
            current_task: None,
            id_counter: AtomicU64::new(1),
            tsc_deadline,
            lapic_freq: distros_cpuid::get_processor_frequency_info()
                .map(|s| s.bus_frequency())
                .unwrap_or(100) as u64
                * 1000
                * 1000
                / Self::DIVIDER,
        }
    }

    pub fn get_state(&self, task: TaskId) -> Option<TaskState> {
        self.task_states.get_state(task)
    }

    pub fn current(&self) -> Option<TaskId> {
        without_interrupts(|| self.current_task.as_ref().map(|s| s.id.clone()))
    }

    pub fn add(
        &self,
        task: Pin<Box<dyn Future<Output = ()>>>,
        nice_level: NiceLevel,
        flags: TaskFlags,
    ) -> TaskId {
        let id = self.id_counter.fetch_add(1, Ordering::SeqCst);
        without_interrupts(|| {
            let mut tasks = self.waiting_tasks.lock();
            tasks.insert(Box::new(WaitingTask {
                id: TaskId(id),
                state: WaitingTaskState::Ready(task),
                run_time: tsc(),
                nice: nice_level,
                link: Default::default(),
                flags,
                buffer_handle: None,
            }));
        });
        TaskId(id)
    }

    fn setup_timer(&self, deadline: Duration) {
        if self.tsc_deadline {
            distros_interrupt_pic::lapic_timer_set_tsc_deadline(distros_timer_tsc::tsc_cycles(
                deadline,
            ));
        } else {
            distros_interrupt_pic::lapic_timer_add_initial(
                (self.lapic_freq / (1000_000 / deadline.as_micros()) as u64) as u32,
            )
        }
    }

    pub unsafe fn int(&mut self, stack_frame: &mut InterruptStackFrame, regs: &mut Regs) {
        distros_interrupt_pic::lapic_timer_disable();
        warn!("A");
        if let Some(mut task) = self.current_task.take() {
            if task.flags.contains(TaskFlags::NOPREEMPT) {
                distros_interrupt_pic::lapic_eoi();
                x86_64::instructions::interrupts::enable();
                return;
            }

            let stack = if let Some(stack) = task.stack_handle {
                stack
            } else {
                let (mut buffer, handle) =
                    distros_memory_stack::make_copy(task.id.0, &StackBuffer::KernelBase);
                let delta = regs.rdi - regs.rsi;
                regs.rsi = buffer.start().as_u64();
                regs.rdi = buffer.start().as_u64() + delta;
                handle
            };
            warn!("BCTX");
            let ctx = TaskContext::fill_from(&stack_frame, regs);
            warn!("CTX");
            {
                let mut tasks = self.waiting_tasks.lock();
                tasks.insert(Box::new(WaitingTask {
                    run_time: tsc(),
                    state: WaitingTaskState::Paused(ctx.clone()),
                    link: RBTreeLink::new(),
                    id: task.id,
                    nice: task.nice,
                    flags: task.flags,
                    buffer_handle: Some(stack),
                }));
            }
            self.task_states.set_state(task.id, TaskState::Waiting);
        }

        loop {
            let task = {
                let mut tasks = self.waiting_tasks.lock();
                tasks.front_mut().remove()
            };
            match task {
                None => {
                    distros_interrupt_pic::lapic_eoi();
                    x86_64::instructions::interrupts::enable();
                    x86_64::instructions::hlt();
                }
                Some(task) => {
                    self.current_task = Some(RunningTask {
                        id: task.id,
                        nice: task.nice,
                        flags: task.flags,
                        stack_handle: task.buffer_handle,
                    });
                    match task.state {
                        WaitingTaskState::Paused(ctx) => {
                            // if paused once - cannot be NOPREEMPT
                            self.task_states.set_state(task.id, TaskState::Running);
                            let deadline = DEADLINE - NICE_PERIOD * task.nice.level() as u32;
                            self.setup_timer(deadline);
                            ctx.save_info(stack_frame.as_mut().extract_inner(), regs);
                            distros_interrupt_pic::lapic_timer_enable();
                            distros_interrupt_pic::lapic_eoi();
                            x86_64::instructions::interrupts::enable();
                            warn!("N");
                            return;
                        }
                        WaitingTaskState::Ready(mut future) => {
                            self.task_states.set_state(task.id, TaskState::Running);
                            let waker = Arc::new(TaskWaker::new(&self.waiting_tasks));
                            if !task.flags.contains(TaskFlags::NOPREEMPT) {
                                let deadline = DEADLINE - NICE_PERIOD * task.nice.level() as u32;
                                self.setup_timer(deadline);
                                distros_interrupt_pic::lapic_timer_enable();
                            }
                            distros_interrupt_pic::lapic_eoi();
                            x86_64::instructions::interrupts::enable();
                            let result = future
                                .as_mut()
                                .poll(&mut Context::from_waker(&waker.clone().into()));
                            x86_64::instructions::interrupts::disable();
                            distros_interrupt_pic::lapic_timer_disable();
                            match result {
                                Poll::Ready(_) => self.task_states.remove_state(task.id),
                                Poll::Pending => {
                                    let taken_current = self.current_task.take().unwrap();
                                    let ntask = WaitingTask {
                                        id: task.id,
                                        nice: task.nice,
                                        link: RBTreeLink::new(),
                                        state: WaitingTaskState::Ready(future),
                                        run_time: tsc(),
                                        flags: task.flags,
                                        buffer_handle: taken_current.stack_handle,
                                    };
                                    if waker.wake_called.load(Ordering::SeqCst) {
                                        let mut queue = self.waiting_tasks.lock();
                                        queue.insert(Box::new(ntask));
                                    } else {
                                        {
                                            let mut waker_task = waker.task.lock();
                                            *waker_task = Some(ntask);
                                        }
                                        self.task_states.set_state(task.id, TaskState::Parked)
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
