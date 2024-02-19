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
#![feature(allocator_api)]
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
use core::time::Duration;

use crate::gui::TextDisplay;
use bootloader_api::config::Mapping;
use bootloader_api::{entry_point, BootInfo, BootloaderConfig};
use chrono::NaiveDateTime;
use distros_framebuffer_vesa::VesaFrameBuffer;
use distros_logging::Logger;
use log::LevelFilter;
use x86_64::instructions::hlt;
use x86_64::VirtAddr;

#[macro_use]
mod interrupts;
#[macro_use]
mod process;
mod basic_term;
#[macro_use]
mod flow;
mod driver;
mod elf;
mod gui;
mod random;

/// This function is called on panic.
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    Logger::<TextDisplay<VesaFrameBuffer>>::panic(info);
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
    Logger::new(TextDisplay::new(fb))
        .set_max_level(LevelFilter::Debug)
        .init();

    distros_cpuid::load();
    distros_interrupt::init();
    distros_memory::init(
        boot_info.physical_memory_offset.into_option(),
        &boot_info.memory_regions,
    );
    distros_acpi::init_acpi(boot_info.rsdp_addr.into_option());
    distros_interrupt_pic::init();
    distros_timer::init();
    // distros_fpu::init();
    x86_64::instructions::interrupts::enable();
    distros_timer::after_interrupt_enabled();

    loop {
        warn!(
            "TIME {}",
            NaiveDateTime::from_timestamp_millis(distros_timer::now() as i64).unwrap()
        );
        distros_timer::sleep(Duration::from_secs(1));
    }
    // interrupts::syscall_init();
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
        hlt();
    }
}
