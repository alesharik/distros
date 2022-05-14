use bootloader::bootinfo::MemoryMap;
use x86_64::structures::paging::mapper::MapToError;
use x86_64::structures::paging::page::NotGiantPageSize;
use x86_64::structures::paging::Size4KiB;
use x86_64::{structures::paging::Page, PhysAddr, VirtAddr};

use crate::kblog;

mod frame;
mod kheap;
mod page_table;
mod process;
pub mod util;

use core::sync::atomic::{AtomicU64, Ordering};
pub use kheap::{init_kheap, init_kheap_info};
pub use process::{Liballoc, PageAllocator, PageAllocatorBackup};

pub trait AllocatePage<T: NotGiantPageSize = Size4KiB> {
    fn allocate(page: Page<T>) -> Result<(), MapToError<T>>;
}

static PHYS_OFFSET: AtomicU64 = AtomicU64::new(0);

pub fn init_memory(phys_offset: VirtAddr, memory_map: &'static MemoryMap) {
    page_table::init(phys_offset);
    let kernel_heap_info = kheap::init_kheap(memory_map).unwrap();
    PHYS_OFFSET.store(phys_offset.as_u64(), Ordering::SeqCst);
    frame::init(memory_map, &kernel_heap_info.offsets);
    kblog!("MemoryManager", "Memory manager ready");
}

pub fn map_physical_address(address: PhysAddr) -> VirtAddr {
    VirtAddr::new(address.as_u64() + PHYS_OFFSET.load(Ordering::SeqCst))
}
