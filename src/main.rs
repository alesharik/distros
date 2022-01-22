#![feature(asm)]
#![feature(abi_x86_interrupt)]
#![no_std]
#![no_main]
#![feature(alloc_error_handler)]
#![feature(inline_const)]
#![feature(ptr_metadata)]
#![feature(core_intrinsics)]
#![feature(slice_group_by)]
#![feature(linked_list_cursors)]
#![allow(dead_code)]

#[macro_use]
extern crate alloc;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate bitflags;
#[macro_use]
extern crate log;
#[macro_use]
extern crate libkernel;

use core::panic::PanicInfo;

use crate::elf::ElfProgram;
use bootloader::{entry_point, BootInfo};
use x86_64::VirtAddr;

#[macro_use]
mod vga;
mod gdt;
#[macro_use]
mod interrupts;
mod memory;
#[macro_use]
mod futures;
mod acpi;
mod basic_term;
mod cmos;
mod cpuid;
#[macro_use]
mod flow;
mod driver;
mod elf;
mod fpu;
mod random;
mod process;

/// This function is called on panic.
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    eprintln!("{}", info);
    loop {}
}

#[alloc_error_handler]
fn alloc_error_handler(layout: alloc::alloc::Layout) -> ! {
    panic!("allocation error: {:?}", layout)
}

entry_point!(main);

pub fn main(boot_info: &'static BootInfo) -> ! {
    cpuid::init_cpuid();
    gdt::init_gdt();
    interrupts::init_idt();
    memory::init_memory(
        VirtAddr::new(boot_info.physical_memory_offset),
        &boot_info.memory_map,
    );
    let acpi = acpi::init_acpi();
    interrupts::init_pic(&acpi);
    fpu::init_fpu();
    futures::init();
    memory::init_kheap_info();
    interrupts::syscall_init();

    // ElfProgram::load(include_bytes!("../example_elf/target/config/release/example_elf")).unwrap().start_tmp();

    driver::init();
    basic_term::init().unwrap();

    futures::run();
}
