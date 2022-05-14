use crate::memory::process::page::PageAllocator;
use alloc::sync::Arc;
/// Liballoc interface
use core::ffi::c_void;
use spin::Mutex;
use x86_64::VirtAddr;

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

struct LiballocContext {
    page_alloc: Arc<Mutex<PageAllocator>>,
}

extern "C" {
    fn lalloc_malloc(
        heap: *mut process_heap_inner,
        size: usize,
        page_alloc: *mut c_void,
    ) -> *mut c_void;
    fn lalloc_realloc(
        heap: *mut process_heap_inner,
        ptr: *mut c_void,
        size: usize,
        page_alloc: *mut c_void,
    ) -> *mut c_void;
    fn lalloc_calloc(
        heap: *mut process_heap_inner,
        count: usize,
        size: usize,
        page_alloc: *mut c_void,
    ) -> *mut c_void;
    fn lalloc_free(heap: *mut process_heap_inner, ptr: *mut c_void, page_alloc: *mut c_void);
}

#[no_mangle]
unsafe fn liballoc_alloc(
    heap: *mut process_heap_inner,
    size: usize,
    page_alloc: *mut c_void,
) -> *mut c_void {
    let mut inner = &mut *(page_alloc as *mut PageAllocator);
    let mut pages = size / 4096;
    if pages * 4096 != size {
        pages += 1;
    }
    let ret = match inner.allocate(pages as u64) {
        None => 0 as *mut c_void,
        Some(mem) => mem.as_mut_ptr::<c_void>(),
    };
    ret
}

#[no_mangle]
unsafe fn liballoc_free(
    heap: *mut process_heap_inner,
    ptr: *mut c_void,
    size: usize,
    page_alloc: *mut c_void,
) -> i32 {
    let mut inner = &mut *(page_alloc as *mut PageAllocator);
    inner.deallocate(VirtAddr::from_ptr(ptr));
    core::mem::forget(inner);
    0
}

pub struct Liballoc {
    heap: process_heap_inner,
}

impl Liballoc {
    pub fn new() -> Liballoc {
        Liballoc {
            heap: process_heap_inner {
                root: 0 as *mut liballoc_major_block,
                best_bet: 0 as *mut liballoc_major_block,
                allocated: 0,
                inuse: 0,
                error_count: 0,
                possible_overruns: 0,
                warning_count: 0,
            },
        }
    }

    pub fn malloc(&mut self, page_alloc: &mut PageAllocator, size: usize) -> *mut c_void {
        unsafe {
            lalloc_malloc(
                &mut self.heap as *mut process_heap_inner,
                size,
                page_alloc as *mut PageAllocator as *mut _,
            )
        }
    }

    pub fn calloc(
        &mut self,
        page_alloc: &mut PageAllocator,
        count: usize,
        size: usize,
    ) -> *mut c_void {
        unsafe {
            lalloc_calloc(
                &mut self.heap as *mut process_heap_inner,
                count,
                size,
                page_alloc as *mut PageAllocator as *mut _,
            )
        }
    }

    pub fn realloc(
        &mut self,
        page_alloc: &mut PageAllocator,
        ptr: *mut c_void,
        size: usize,
    ) -> *mut c_void {
        unsafe {
            lalloc_realloc(
                &mut self.heap as *mut process_heap_inner,
                ptr,
                size,
                page_alloc as *mut PageAllocator as *mut _,
            )
        }
    }

    pub fn free(&mut self, page_alloc: &mut PageAllocator, ptr: *mut c_void) {
        unsafe {
            lalloc_free(
                &mut self.heap as *mut process_heap_inner,
                ptr,
                page_alloc as *mut PageAllocator as *mut _,
            )
        }
    }
}
