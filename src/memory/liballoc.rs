/// Liballoc interface
use core::ffi::c_void;
use alloc::vec::Vec;
use x86_64::{PhysAddr, VirtAddr};
use x86_64::structures::paging::{PhysFrame, Size4KiB, Page, PageTableFlags};
use crate::memory::process::ProcessMappingInfo;
use crate::memory::frame;
use crate::memory::page_table;

//noinspection RsStructNaming
#[repr(C)]
struct liballoc_major_block {
    prev: *mut liballoc_major_block,
    next: *mut liballoc_major_block,
    pages: u32,
    size: u64,
    usage: u64,
    first: *mut liballoc_minor_block,
}

//noinspection RsStructNaming
#[repr(C)]
struct liballoc_minor_block {
    prev: *mut liballoc_minor_block,
    next: *mut liballoc_minor_block,
    block: *mut liballoc_major_block,
    magic: u32,
    size: u64,
    req_size: u64,
}

//noinspection RsStructNaming
#[repr(C)]
pub struct process_heap_inner {
    root: *mut liballoc_major_block,
    best_bet: *mut liballoc_major_block,
    allocated: u64,
    inuse: u64,
    warning_count: u64,
    error_count: u64,
    possible_overruns: u64,
}

extern "C" {
    fn lalloc_malloc(heap: *mut process_heap_inner, size: usize, inner: *mut c_void) -> *mut c_void;
    fn lalloc_realloc(heap: *mut process_heap_inner, ptr: *mut c_void, size: usize, inner: *mut c_void) -> *mut c_void;
    fn lalloc_calloc(heap: *mut process_heap_inner, count: usize, size: usize, inner: *mut c_void) -> *mut c_void;
    fn lalloc_free(heap: *mut process_heap_inner, ptr: *mut c_void, inner: *mut c_void);
}

#[no_mangle]
unsafe fn liballoc_alloc(size: usize) -> *mut c_void {
    // let mut inner = &*(inner as *mut ProcessMappingInfo);
    // let frame = frame::with_frame_alloc(|a| {
    //     let mut frames  = size / 4096;
    //     if frames * 4096 != size {
    //         frames += 1;
    //     }
    //     a.allocate(frames as u32)
    // });
    // if let Some(frame) = frame {
    //     let addr = inner.allocate_virt(size);
    //     page_table::map(frame, Page::containing_address(addr), PageTableFlags::PRESENT | PageTableFlags::WRITABLE);
    //     addr.as_mut_ptr()
    // } else {
        0 as *mut c_void
    // }
}

#[no_mangle]
unsafe fn liballoc_free(ptr: *mut c_void, size: usize) -> i32 {
    // let mut inner = &*(inner as *mut ProcessMappingInfo);
    // let virt = VirtAddr::new(ptr as u64);
    // let frame = match page_table::unmap(Page::containing_address(virt)) {
    //     Ok(f) => f,
    //     Err(e) => return 1,
    // };
    // inner.deallocate_virt(virt, size);
    // frame::with_frame_alloc(|a| {
    //     a.deallocate(frame);
    // });
    0
}
