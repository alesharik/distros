#![no_std]
#![feature(asm_const)]
#![feature(abi_x86_interrupt)]

#[macro_use]
mod macros;
mod gdt;
mod idt;
mod nmi;

pub use idt::{alloc_handler, has_handler, set_handler, OverrideMode};
pub use nmi::without_nmi;

#[derive(Clone, Copy, Eq, PartialEq, Debug, Ord, PartialOrd)]
#[repr(transparent)]
pub struct InterruptId(usize);

impl InterruptId {
    pub const fn new(int: usize) -> Self {
        InterruptId(int)
    }

    pub const fn int(&self) -> usize {
        self.0
    }
}

pub fn init() {
    gdt::init_gdt();
    idt::init_idt();
    nmi::nmi_enable();
}
