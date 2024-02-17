#![feature(abi_x86_interrupt)]
#![no_std]
#![no_main]
#![feature(alloc_error_handler)]
#![feature(inline_const)]
#![feature(ptr_metadata)]
#![feature(core_intrinsics)]
#![feature(slice_group_by)]
#![feature(naked_functions)]
#![feature(linked_list_cursors)]
#![feature(asm_const)]
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

use core::fmt::Write;
use core::panic::PanicInfo;

use bootloader_api::{entry_point, BootInfo, BootloaderConfig};
use bootloader_api::config::Mapping;
use x86_64::instructions::hlt;
use x86_64::VirtAddr;
use distros_framebuffer_vesa::VesaFrameBuffer;
use crate::gui::TextDisplay;

#[macro_use]
mod logging;
mod gdt;
#[macro_use]
mod interrupts;
#[macro_use]
mod process;
mod memory;
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
mod gui;

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

pub static BOOTLOADER_CONFIG: BootloaderConfig = {
    let mut config = BootloaderConfig::new_default();
    config.mappings.physical_memory = Some(Mapping::Dynamic);
    config
};

entry_point!(main, config = &BOOTLOADER_CONFIG);

pub fn main(boot_info: &'static mut BootInfo) -> ! {
    let fb = VesaFrameBuffer::new(boot_info.framebuffer.take().unwrap());
    logging::init(fb);
    println!("0x{:08x}", &boot_info.physical_memory_offset.into_option().unwrap());

    cpuid::init_cpuid();
    gdt::init_gdt();
    interrupts::init_idt();
    memory::init_memory(
        VirtAddr::new(boot_info.physical_memory_offset.into_option().unwrap()),
        &boot_info.memory_regions,
    );
    let acpi = acpi::init_acpi(boot_info.rsdp_addr.into_option());
    interrupts::init_pic(&acpi);
    fpu::init_fpu();
    memory::init_kheap_info();
    interrupts::syscall_init();
    // //
    // // // ElfProgram::load(include_bytes!("../example_elf/target/config/release/example_elf")).unwrap().start_tmp();
    //
    // process::setup();
    //
    // driver::init(&acpi);
    // basic_term::init().unwrap();
    //
    // unsafe { process::run() }
    loop {

    }
}
