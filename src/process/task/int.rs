use crate::interrupts::INT_LAPIC_TIMER;
use crate::process::task::ctx::Regs;
use crate::process::task::ProcessRuntime;
use core::arch::asm;
use distros_interrupt::OverrideMode;
use x86_64::structures::idt::InterruptStackFrame;

static mut RUNTIME: Option<ProcessRuntime> = Option::None;
//
// #[naked]
// pub extern "x86-interrupt" fn switch_context(frame: InterruptStackFrame) {
//     unsafe {
//         asm!(
//             "mov     qword ptr [rsp - 120], r15",
//             "mov     qword ptr [rsp - 112], r14",
//             "mov     qword ptr [rsp - 104], r13",
//             "mov     qword ptr [rsp - 96], r12",
//             "mov     qword ptr [rsp - 88], r11",
//             "mov     qword ptr [rsp - 80], r10",
//             "mov     qword ptr [rsp - 72], r9",
//             "mov     qword ptr [rsp - 64], r8",
//             "mov     qword ptr [rsp - 56], rsi",
//             "mov     qword ptr [rsp - 48], rdi",
//             "mov     qword ptr [rsp - 40], rbp",
//             "mov     qword ptr [rsp - 32], rdx",
//             "mov     qword ptr [rsp - 24], rcx",
//             "mov     qword ptr [rsp - 16], rbx",
//             "mov     qword ptr [rsp - 8], rax",
//             "sub     rsp, 168",
//             "lea     rdi, [rsp + 8]",
//             "call    {}",
//             "add     rsp, 168",
//             "mov     r15, qword ptr [rsp - 120]",
//             "mov     r14, qword ptr [rsp - 112]",
//             "mov     r13, qword ptr [rsp - 104]",
//             "mov     r12, qword ptr [rsp - 96]",
//             "mov     r11, qword ptr [rsp - 88]",
//             "mov     r10, qword ptr [rsp - 80]",
//             "mov     r9, qword ptr [rsp - 72]",
//             "mov     r8, qword ptr [rsp - 64]",
//             "mov     rsi, qword ptr [rsp - 56]",
//             "mov     rdi, qword ptr [rsp - 48]",
//             "mov     rbp, qword ptr [rsp - 40]",
//             "mov     rdx, qword ptr [rsp - 32]",
//             "mov     rcx, qword ptr [rsp - 24]",
//             "mov     rbx, qword ptr [rsp - 16]",
//             "mov     rax, qword ptr [rsp - 8]",
//             "iretq",
//             sym switch_context_int,
//             options(noreturn)
//         )
//     }
// }

#[inline]
unsafe extern "C" fn switch_context_int(mut stack_frame: InterruptStackFrame, regs: &mut Regs) {
    if let Some(runtime) = &mut RUNTIME {
        runtime.int(stack_frame, regs)
    }
}

pub fn setup(runtime: ProcessRuntime) {
    unsafe {
        RUNTIME = Some(runtime);
    }

    // distros_interrupt::set_handler(INT_LAPIC_TIMER, switch_context, OverrideMode::Override);
    // crate::interrupts::start_lapic_timer();
}

pub unsafe fn run() -> ! {
    if let Some(runtime) = &mut RUNTIME {
        runtime.run()
    } else {
        panic!("Process runtime not set up");
    }
}
