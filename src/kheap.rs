use crate::memory::{MemoryManager, AllocatePage};
use x86_64::VirtAddr;
use x86_64::structures::paging::{Page, Size2MiB};
use x86_64::structures::paging::mapper::MapToError;
use linked_list_allocator::LockedHeap;
use crate::kblog;

pub const HEAP_START: usize = 0x_4444_4444_0000;
pub const HEAP_SIZE: usize = 4 * 1024 * 1024; // 2 MiB

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

pub fn init_kheap() -> Result<(), MapToError<Size2MiB>> {
    kblog!("KHeap", "Starting kernel heap");
    let heap_start = VirtAddr::new(HEAP_START as u64);
    let heap_end = heap_start + HEAP_SIZE - 1u64;
    let heap_start_page = Page::<Size2MiB>::containing_address(heap_start);
    let heap_end_page = Page::<Size2MiB>::containing_address(heap_end);
    for page in Page::<Size2MiB>::range_inclusive(heap_start_page, heap_end_page) {
        MemoryManager::allocate(page)?
    }

    unsafe {
        ALLOCATOR.lock().init(HEAP_START, HEAP_SIZE);
    }

    kblog!("KHeap", "Kernel heap started at pos {:#x} with size {} MiB", HEAP_START, HEAP_SIZE / 1024 / 1024);
    Ok(())
}