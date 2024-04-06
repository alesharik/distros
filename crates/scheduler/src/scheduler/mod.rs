use crate::scheduler::context::Regs;
use crate::scheduler::logic::Scheduler;
use crate::{NiceLevel, TaskFlags, TaskId};
use alloc::boxed::Box;
use core::arch::asm;
use core::future::Future;
use core::pin::Pin;
use distros_interrupt::OverrideMode;
use distros_timer_tsc::tsc;
use log::debug;
use x2apic::lapic::{TimerDivide, TimerMode};
use x86_64::instructions::hlt;
use x86_64::structures::idt::InterruptStackFrame;

mod context;
mod logic;

static mut SCHED: Option<Scheduler> = None;

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum TaskState {
    Running,
    Waiting,
    Parked,
}

#[naked]
pub extern "x86-interrupt" fn switch_context(frame: InterruptStackFrame) {
    unsafe {
        asm!(
        "mov     qword ptr [rsp - 120], r15",
        "mov     qword ptr [rsp - 112], r14",
        "mov     qword ptr [rsp - 104], r13",
        "mov     qword ptr [rsp - 96], r12",
        "mov     qword ptr [rsp - 88], r11",
        "mov     qword ptr [rsp - 80], r10",
        "mov     qword ptr [rsp - 72], r9",
        "mov     qword ptr [rsp - 64], r8",
        "mov     qword ptr [rsp - 56], rsi",
        "mov     qword ptr [rsp - 48], rdi",
        "mov     qword ptr [rsp - 40], rbp",
        "mov     qword ptr [rsp - 32], rdx",
        "mov     qword ptr [rsp - 24], rcx",
        "mov     qword ptr [rsp - 16], rbx",
        "mov     qword ptr [rsp - 8], rax",
        "sub     rsp, 168",
        "lea     rdi, [rsp + 8]",
        "call    {}",
        "add     rsp, 168",
        "mov     r15, qword ptr [rsp - 120]",
        "mov     r14, qword ptr [rsp - 112]",
        "mov     r13, qword ptr [rsp - 104]",
        "mov     r12, qword ptr [rsp - 96]",
        "mov     r11, qword ptr [rsp - 88]",
        "mov     r10, qword ptr [rsp - 80]",
        "mov     r9, qword ptr [rsp - 72]",
        "mov     r8, qword ptr [rsp - 64]",
        "mov     rsi, qword ptr [rsp - 56]",
        "mov     rdi, qword ptr [rsp - 48]",
        "mov     rbp, qword ptr [rsp - 40]",
        "mov     rdx, qword ptr [rsp - 32]",
        "mov     rcx, qword ptr [rsp - 24]",
        "mov     rbx, qword ptr [rsp - 16]",
        "mov     rax, qword ptr [rsp - 8]",
        "iretq",
        sym switch_context_int,
        options(noreturn)
        )
    }
}

#[inline]
unsafe extern "C" fn switch_context_int(mut stack_frame: InterruptStackFrame, regs: &mut Regs) {
    if let Some(sched) = SCHED.as_mut() {
        sched.int(&mut stack_frame, regs);
    }
    stack_frame.iretq()
}

pub fn init() {
    let has_tsc_deadline = distros_cpuid::get_feature_info().has_tsc_deadline();
    unsafe {
        SCHED = Some(Scheduler::new(has_tsc_deadline));
    }
    distros_interrupt_pic::lapic_timer_disable();
    if has_tsc_deadline {
        debug!("Scheduler will use TSC-deadline mode on LAPIC");
        distros_interrupt_pic::lapic_timer_set_mode(TimerMode::TscDeadline, TimerDivide::Div2);
    } else {
        debug!("Scheduler will use counter on LAPIC");
        distros_interrupt_pic::lapic_timer_set_mode(TimerMode::OneShot, Scheduler::LAPIC_DIVIDER);
    }
    distros_interrupt::set_handler(
        distros_interrupt_pic::INT_LAPIC_TIMER,
        switch_context,
        OverrideMode::Panic,
    );
}

pub fn start() -> ! {
    let has_tsc_deadline = distros_cpuid::get_feature_info().has_tsc_deadline();
    if has_tsc_deadline {
        distros_interrupt_pic::lapic_timer_set_tsc_deadline(tsc());
    } else {
        distros_interrupt_pic::lapic_timer_add_initial(10000);
    }
    distros_interrupt_pic::lapic_timer_enable();
    loop {
        hlt();
    }
}

pub fn add(
    task: Pin<Box<dyn Future<Output = ()>>>,
    nice_level: NiceLevel,
    flags: TaskFlags,
) -> TaskId {
    unsafe {
        let sched = SCHED.as_ref().expect("Scheduler not initialized");
        sched.add(task, nice_level, flags)
    }
}

pub fn get_state(task_id: TaskId) -> Option<TaskState> {
    unsafe {
        let sched = SCHED.as_ref().expect("Scheduler not initialized");
        sched.get_state(task_id)
    }
}

pub fn current() -> Option<TaskId> {
    unsafe {
        let sched = SCHED.as_ref().expect("Scheduler not initialized");
        sched.current()
    }
}
