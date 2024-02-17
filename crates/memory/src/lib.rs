#![no_std]

use bootloader_api::info::MemoryRegions;
use log::info;
use x86_64::{PhysAddr, VirtAddr};

pub mod arena;
mod frame_alloc;
mod kalloc;
mod page_table;

pub use page_table::{map, unmap};

static mut PHYS_OFFSET: u64 = 0;

pub fn init(offset: Option<u64>, regions: &MemoryRegions) {
    unsafe {
        PHYS_OFFSET = offset.expect("Kernel required to start with physical memory already mapped");
        info!("Physical memory offset = 0x{:08x}", PHYS_OFFSET);
        arena::initialize(regions);
        page_table::init(VirtAddr::new(PHYS_OFFSET));
    }
}

pub fn translate_kernel(phys: PhysAddr) -> VirtAddr {
    unsafe { VirtAddr::new_truncate((phys + PHYS_OFFSET).as_u64()) }
}
