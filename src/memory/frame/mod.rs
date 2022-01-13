//! This module manages physical frames and frame regions

use alloc::collections::LinkedList;
use alloc::rc::Rc;
use arrayvec::ArrayVec;
use bootloader::bootinfo::{MemoryMap, MemoryRegion, MemoryRegionType};
use core::cell::RefCell;
use core::ops::Not;
use spin::Mutex;
use x86_64::structures::paging::{
    FrameAllocator, FrameDeallocator, PageSize, PhysFrame, Size2MiB, Size4KiB,
};
use x86_64::PhysAddr;
use crate::memory::frame::region::MemoryRegionProvider;

mod region;

/// Frame region representation
#[derive(Clone)]
#[repr(C)]
struct Frame {
    /// Start frame of the region
    start_frame: PhysAddr,
    /// Size in page length
    /// last bit - used(1)/free(0)
    size: u64
}

impl Frame {
    /// create new frame
    fn new(start: PhysAddr, size: u64, used: bool) -> Self {
        let mut frame = Frame {
            start_frame: start,
            size,
        };
        frame.set_used(used);
        frame
    }

    /// Get frame size
    fn get_size(&self) -> u64 {
        self.size & ((1u64 << 63) as u64).not()
    }

    /// Get frame used flag
    fn is_used(&self) -> bool {
        self.size & ((1u64 << 63) as u64) == ((1u64 << 63) as u64)
    }

    /// Set frame size
    fn set_size(&mut self, size: u64) {
        self.size = size | (self.size & ((1u64 << 63) as u64))
    }

    /// Set frame used flag
    fn set_used(&mut self, used: bool) {
        if used {
            self.size |= (1u64 << 63) as u64
        } else {
            self.size &= ((1u64 << 63) as u64).not()
        }
    }

    /// Slice current frame in two at point
    fn slice(&mut self, rem_size: u64) -> Frame {
        let ret = Frame::new(self.start_frame + rem_size, rem_size, false);
        self.set_size(self.size - rem_size);
        ret
    }
}

pub struct FrameAlloc {
    region_provider: MemoryRegionProvider,
    frames: LinkedList<Frame>,
}

impl FrameAlloc {
    fn new(memory_map: &'static MemoryMap, offsets: &[u64]) -> FrameAlloc {
        FrameAlloc {
            region_provider: MemoryRegionProvider::new(memory_map, offsets),
            frames: LinkedList::new()
        }
    }

    pub fn allocate(&mut self, frames: u64) -> Option<PhysFrame<Size4KiB>> {
        let mut cursor = self.frames.cursor_front_mut();
        while let Some(frame) = cursor.current() {
            if frame.is_used() || frame.get_size() < frames {
                cursor.move_next();
                continue;
            }
            frame.set_used(true);
            let start = frame.start_frame;
            let diff = frame.get_size() - frames;
            if diff > 0 {
                let trailing = frame.slice(diff);
                cursor.insert_after(trailing);
            }
            return Some(PhysFrame::containing_address(start));
        }
        if let Some(addr) = self.region_provider.take(frames as u64) {
            let frame = Frame::new(addr, frames as u64, true);
            self.frames.push_back(frame);
            return Some(PhysFrame::containing_address(addr));
        }
        None
    }

    pub fn deallocate(&mut self, address: PhysFrame<Size4KiB>) {
        let mut cursor = self.frames.cursor_front_mut();
        while let Some(frame) = cursor.current() {
            if frame.start_frame == address.start_address() {
                // we found current frame
                frame.set_used(false);

                // merge all free frames after current
                cursor.move_next();
                while let Some(frame) = cursor.current() {
                    if frame.is_used() {
                        break;
                    }
                    frame.size += frame.size;
                    cursor.remove_current();
                }
                return;
            } else {
                cursor.move_next();
            }
        }
    }
}

unsafe impl FrameAllocator<Size4KiB> for FrameAlloc {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        self.allocate(1)
    }
}

unsafe impl FrameAllocator<Size2MiB> for FrameAlloc {
    fn allocate_frame(&mut self) -> Option<PhysFrame<Size2MiB>> {
        self.allocate(512).map(|e| unsafe {
            PhysFrame::<Size2MiB>::from_start_address_unchecked(e.start_address())
        })
    }
}

impl FrameDeallocator<Size4KiB> for FrameAlloc {
    unsafe fn deallocate_frame(&mut self, frame: PhysFrame) {
        self.deallocate(frame)
    }
}

impl FrameDeallocator<Size2MiB> for FrameAlloc {
    unsafe fn deallocate_frame(&mut self, frame: PhysFrame<Size2MiB>) {
        self.deallocate(PhysFrame::from_start_address_unchecked(
            frame.start_address(),
        ))
    }
}

unsafe impl Send for FrameAlloc {}

lazy_static! {
    static ref MEMORY_FRAME_ALLOCATOR: Mutex<Option<FrameAlloc>> = Mutex::new(None);
}

pub fn init(memory_map: &'static MemoryMap, offsets: &[u64]) {
    let mut alloc = MEMORY_FRAME_ALLOCATOR.lock();
    *alloc = Some(FrameAlloc::new(memory_map, offsets));
}

pub fn with_frame_alloc<T, F: Fn(&mut FrameAlloc) -> T>(function: F) -> T {
    crate::interrupts::no_int(|| {
        let mut alloc = MEMORY_FRAME_ALLOCATOR.lock();
        let alloc = alloc.as_mut().unwrap();
        function(alloc)
    })
}
