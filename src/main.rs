#![feature(asm)]
#![feature(abi_x86_interrupt)]
#![no_std]
#![no_main]
#![feature(alloc_error_handler)]
#![allow(dead_code)]

extern crate alloc;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate bitflags;

use core::panic::PanicInfo;
use core::time::Duration;

use bootloader::{BootInfo, entry_point};
use x86_64::VirtAddr;

#[macro_use]
mod vga;
mod gdt;
#[macro_use]
mod interrupts;
mod memory;
mod acpi;
mod cpuid;
mod cmos;
mod random;
mod fpu;
mod flow;

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
    memory::init_memory(VirtAddr::new(boot_info.physical_memory_offset), &boot_info.memory_map);
    memory::print_table();
    memory::init_kheap().unwrap();
    let acpi = acpi::init_acpi();
    interrupts::init_pic(&acpi);
    fpu::init_fpu();

    println!("TIMEOUT");
    interrupts::sleep(Duration::from_secs(2));
    println!("TIMEOUT");
    println!("TIME {}", cmos::read_time());

    loop {
        x86_64::instructions::hlt();
    }
}