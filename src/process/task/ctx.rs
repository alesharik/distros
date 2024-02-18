use distros_fpu::FpuState;
use x86_64::structures::idt::{InterruptStackFrame, InterruptStackFrameValue};
use x86_64::VirtAddr;

#[repr(C)]
#[derive(Debug)]
pub struct Regs {
    pub r15: u64,
    pub r14: u64,
    pub r13: u64,
    pub r12: u64,
    pub r11: u64,
    pub r10: u64,
    pub r9: u64,
    pub r8: u64,
    pub rsi: u64,
    pub rdi: u64,
    pub rbp: u64,
    pub rdx: u64,
    pub rcx: u64,
    pub rbx: u64,
    pub rax: u64,
}

impl Regs {
    pub fn take_from(&mut self, p0: &Regs) {
        self.r15 = p0.r15;
        self.r14 = p0.r14;
        self.r13 = p0.r13;
        self.r12 = p0.r12;
        self.r11 = p0.r11;
        self.r10 = p0.r10;
        self.r9 = p0.r9;
        self.r8 = p0.r8;
        self.rsi = p0.rsi;
        self.rdi = p0.rdi;
        self.rbp = p0.rbp;
        self.rdx = p0.rdx;
        self.rcx = p0.rcx;
        self.rbx = p0.rbx;
        self.rax = p0.rax;
    }

    pub fn put_into(&self, p0: &mut Regs) {
        p0.r15 = self.r15;
        p0.r14 = self.r14;
        p0.r13 = self.r13;
        p0.r12 = self.r12;
        p0.r11 = self.r11;
        p0.r10 = self.r10;
        p0.r9 = self.r9;
        p0.r8 = self.r8;
        p0.rsi = self.rsi;
        p0.rdi = self.rdi;
        p0.rbp = self.rbp;
        p0.rdx = self.rdx;
        p0.rcx = self.rcx;
        p0.rbx = self.rbx;
        p0.rax = self.rax;
    }
}

impl Regs {
    const fn new() -> Self {
        Regs {
            r15: 0,
            r14: 0,
            r13: 0,
            r12: 0,
            r11: 0,
            r10: 0,
            r9: 0,
            r8: 0,
            rsi: 0,
            rdi: 0,
            rbp: 0,
            rdx: 0,
            rcx: 0,
            rbx: 0,
            rax: 0,
        }
    }
}

pub struct TaskContext {
    pub regs: Regs,
    pub fpu: FpuState,
    pub instruction_pointer: VirtAddr,
    pub stack_pointer: VirtAddr,
    pub code_segment: u64,
    pub stack_segment: u64,
}

impl TaskContext {
    pub const fn new() -> Self {
        TaskContext {
            regs: Regs::new(),
            fpu: FpuState::new(),
            instruction_pointer: VirtAddr::new_truncate(0),
            stack_pointer: VirtAddr::new_truncate(0),
            code_segment: 0,
            stack_segment: 0,
        }
    }

    pub unsafe fn fill_from(frame: &InterruptStackFrame, regs: &Regs) -> TaskContext {
        let mut ctx = TaskContext::new();
        ctx.stack_pointer = frame.stack_pointer;
        ctx.instruction_pointer = frame.instruction_pointer;
        ctx.stack_segment = frame.stack_segment;
        ctx.code_segment = frame.code_segment;
        ctx.fpu.save();
        ctx.regs.take_from(regs);
        ctx
    }

    pub unsafe fn save_info(&self, frame: &mut InterruptStackFrameValue, regs: &mut Regs) {
        self.fpu.restore();
        self.regs.put_into(regs);
        frame.instruction_pointer = self.instruction_pointer;
        frame.stack_pointer = self.stack_pointer;
        frame.code_segment = self.code_segment;
        frame.stack_segment = self.stack_segment;
    }
}
