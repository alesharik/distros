use x86_64::structures::idt::InterruptStackFrame;
use crate::sched::ctx::{Regs, TaskContext};
use crate::fpu::FpuState;
use crate::sched::{SCHEDULER, Task};

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
            sym switch_context_int,
        )
    }
}

static mut CURRENT_TASK: Option<Task> = Some(Task::new());

#[inline]
unsafe extern "C" fn switch_context_int(mut stack_frame: InterruptStackFrame, regs: &mut Regs) {
    let task = CURRENT_TASK.take();
    let mut task = if let Some(task) = task { task } else { return };
    task.context.stack_pointer = stack_frame.stack_pointer;
    task.context.instruction_pointer = stack_frame.instruction_pointer;
    task.context.stack_segment = stack_frame.stack_segment;
    task.context.code_segment = stack_frame.code_segment;
    task.context.fpu.save();
    task.context.regs.take_from(regs);

    let task = SCHEDULER.next_task(task);

    task.context.fpu.restore();
    task.context.regs.put_into(regs);
    let frame = stack_frame.as_mut().extract_inner();
    frame.instruction_pointer = task.context.instruction_pointer;
    frame.stack_pointer = task.context.stack_pointer;
    frame.code_segment = task.context.code_segment;
    frame.stack_segment = task.context.stack_segment;
    CURRENT_TASK = Some(task);
    crate::interrupts::eoi();
}