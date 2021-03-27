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
use bootloader::{BootInfo, entry_point};
use x86_64::VirtAddr;
use core::time::Duration;

#[macro_use]
mod vga;
mod gdt;
mod interrupts;
mod memory;
mod kheap;
mod acpi;
mod cpuid;
mod pic;
mod cmos;
mod timer;
mod random;

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
    println!("Hello World{}", "!");
    cpuid::init_cpuid();
    gdt::init_gdt();
    interrupts::init_idt();
    memory::init_memory(VirtAddr::new(boot_info.physical_memory_offset), &boot_info.memory_map);
    memory::print_table();
    kheap::init_kheap().unwrap();
    let acpi = acpi::init_acpi();

    pic::disable_interrupts();

    pic::init_pic(&acpi.apic);
    timer::init_timer();

    pic::enable_interrupts();

    println!("TIMEOUT");
    timer::sleep(Duration::from_secs(2));
    println!("TIMEOUT");
    println!("TIME {}", cmos::read_time());

    loop {
        x86_64::instructions::hlt();
    }
}