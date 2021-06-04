use crate::memory::{MemoryManager, AllocatePage};
use x86_64::VirtAddr;
use x86_64::structures::paging::{Page, Size2MiB};
use x86_64::structures::paging::mapper::MapToError;
use linked_list_allocator::Heap;
use crate::kblog;
use core::ops::Deref;
use spin::Mutex;
use core::alloc::{GlobalAlloc, Layout};
use core::ptr::NonNull;
use crate::interrupts;

pub const HEAP_START: usize = 0x_4444_4444_0000;
pub const HEAP_SIZE: usize = 4 * 1024 * 1024; // 2 MiB

pub struct LockedHeap(Mutex<Heap>);

impl LockedHeap {
    /// Creates an empty heap. All allocate calls will return `None`.
    pub const fn empty() -> LockedHeap {
        LockedHeap(Mutex::new(Heap::empty()))
    }

    /// Creates a new heap with the given `bottom` and `size`. The bottom address must be valid
    /// and the memory in the `[heap_bottom, heap_bottom + heap_size)` range must not be used for
    /// anything else. This function is unsafe because it can cause undefined behavior if the
    /// given address is invalid.
    pub unsafe fn new(heap_bottom: usize, heap_size: usize) -> LockedHeap {
        LockedHeap(Mutex::new(Heap::new(heap_bottom, heap_size)))
    }
}

impl Deref for LockedHeap {
    type Target = Mutex<Heap>;

    fn deref(&self) -> &Mutex<Heap> {
        &self.0
    }
}

unsafe impl GlobalAlloc for LockedHeap {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        interrupts::no_int(|| {
            self.0
                .lock()
                .allocate_first_fit(layout)
                .ok()
                .map_or(0 as *mut u8, |allocation| allocation.as_ptr())
        })
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        interrupts::no_int(|| {
            self.0
                .lock()
                .deallocate(NonNull::new_unchecked(ptr), layout)
        })
    }
}

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