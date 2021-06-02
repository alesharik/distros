/// Liballoc interface
#![allow(non_camel_case_types)]

use core::ffi::c_void;
use crate::memory::MemoryManager;
use x86_64::structures::paging::Page;
use x86_64::VirtAddr;
use alloc::boxed::Box;

#[repr(C)]
struct liballoc_major_block {
    prev: *mut liballoc_major_block,
    next: *mut liballoc_major_block,
    pages: u32,
    size: u64,
    usage: u64,
    first: *mut liballoc_minor_block,
}

#[repr(C)]
struct liballoc_minor_block {
    prev: *mut liballoc_minor_block,
    next: *mut liballoc_minor_block,
    block: *mut liballoc_major_block,
    magic: u32,
    size: u64,
    req_size: u64,
}

#[repr(C)]
struct process_heap_inner {
    root: *mut liballoc_major_block,
    best_bet: *mut liballoc_major_block,
    allocated: u64,
    inuse: u64,
    warning_count: u64,
    error_count: u64,
    possible_overruns: u64,
}

extern "C" {
    fn lalloc_malloc(heap: *mut process_heap_inner, size: usize) -> *mut c_void;
    fn lalloc_realloc(heap: *mut process_heap_inner, ptr: *mut c_void, size: usize) -> *mut c_void;
    fn lalloc_calloc(heap: *mut process_heap_inner, count: usize, size: usize) -> *mut c_void;
    fn lalloc_free(heap: *mut process_heap_inner, ptr: *mut c_void);
}

#[no_mangle]
fn liballoc_alloc(size: usize) -> *mut c_void {
    0 as *mut c_void
}

#[no_mangle]
fn liballoc_free(ptr: *mut c_void, size: usize) -> i32 {
    // MemoryManager::deallocate_frames(Page::from_start_address(VirtAddr::from_ptr(ptr)).unwrap());
    0
}