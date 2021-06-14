use alloc::rc::Rc;
use alloc::vec::Vec;
/// Main physical page manager
use bootloader::bootinfo::{MemoryMap, MemoryRegion};
use core::cell::RefCell;
use core::ops::Not;
use x86_64::structures::paging::{FrameAllocator, FrameDeallocator, PhysFrame, Size2MiB, Size4KiB};
use x86_64::PhysAddr;

const FRAME_LENGTH: u32 = 4096;

#[derive(Clone)]
#[repr(C)]
struct Frame {
    start_frame: PhysFrame<Size4KiB>,
    /// size in page length
    /// last bit - used(1)/free(0)
    size: u64,
    next: Option<Rc<RefCell<Frame>>>,
}

struct MemoryRegionContainer {
    end: u64,
    pointer: u64,
}

impl MemoryRegionContainer {
    fn new(region: &MemoryRegion) -> Self {
        MemoryRegionContainer {
            end: region.range.end_frame_number,
            pointer: region.range.start_frame_number,
        }
    }

    fn take(&mut self, frames: u64) -> Option<PhysFrame<Size4KiB>> {
        if self.end - self.pointer >= FRAME_LENGTH as u64 * frames {
            let frame = PhysFrame::from_start_address(PhysAddr::new(self.pointer)).unwrap();
            self.pointer += FRAME_LENGTH as u64 * frames;
            Some(frame)
        } else {
            None
        }
    }
}

impl Frame {
    fn new(start: PhysFrame<Size4KiB>, size: u64, used: bool) -> Self {
        let mut frame = Frame {
            start_frame: start,
            size,
            next: None,
        };
        frame.set_used(used);
        frame
    }

    fn get_size(&self) -> u64 {
        self.size & ((1u64 << 63) as u64).not()
    }

    fn is_used(&self) -> bool {
        self.size & ((1u64 << 63) as u64) == ((1u64 << 63) as u64)
    }

    fn set_size(&mut self, size: u64) {
        self.size = size | (self.size & ((1u64 << 63) as u64))
    }

    fn set_used(&mut self, used: bool) {
        if used {
            self.size |= ((1u64 << 63) as u64)
        } else {
            self.size &= ((1u64 << 63) as u64).not()
        }
    }
}

pub struct FrameAlloc {
    regions: Vec<MemoryRegionContainer>,
    root: Rc<RefCell<Frame>>,
}

impl FrameAlloc {
    pub fn new(memory_map: &'static MemoryMap, offset: u32) -> FrameAlloc {
        let mut alloc = FrameAlloc {
            regions: memory_map
                .iter()
                .map(|r| MemoryRegionContainer::new(r))
                .collect::<Vec<_>>(),
            root: Rc::new(RefCell::new(Frame::new(
                PhysFrame::from_start_address(PhysAddr::new(0)).unwrap(),
                0,
                true,
            ))),
        };
        alloc.allocate(offset / FRAME_LENGTH);
        alloc
    }

    pub fn allocate(&mut self, frames: u32) -> Option<PhysFrame<Size4KiB>> {
        let mut it = Some(self.root.clone());
        while !matches!(it, None) {
            let arc = it.unwrap();
            let mut contents = arc.borrow_mut();
            if !contents.is_used() && contents.get_size() >= frames as u64 {
                let diff = contents.get_size() - frames as u64;
                if diff > 0 {
                    let mut remaining_space = Frame::new(contents.start_frame + diff, diff, false);
                    remaining_space.next = contents.next.clone();
                    contents.next = Some(Rc::new(RefCell::new(remaining_space)));
                    contents.set_size(frames as u64);
                }
                contents.set_used(true);
                return Some(contents.start_frame);
            }
            if matches!(contents.next, None) {
                // last iteration
                for reg in &mut self.regions {
                    if let Some(phys) = reg.take(frames as u64) {
                        let frame = Frame::new(phys, frames as u64, true);
                        contents.next = Some(Rc::new(RefCell::new(frame)));
                        return Some(phys);
                    }
                }
            }
            it = contents.next.clone();
        }
        None
    }

    pub fn deallocate(&mut self, address: PhysFrame<Size4KiB>) {
        let mut it = Some(self.root.clone());
        while !matches!(it, None) {
            let arc = it.unwrap();
            let mut contents = arc.borrow_mut();
            if contents.start_frame == address {
                // we found current frame
                contents.set_used(false);

                // merge all free frames after current
                let mut next_it = contents.next.clone();
                loop {
                    if let Some(next) = next_it.clone() {
                        let next = next.borrow();
                        if next.is_used() {
                            return;
                        }
                        contents.size += next.size;
                        next_it = next.next.clone();
                        contents.next = next_it.clone()
                    } else {
                        return; // got end of list
                    }
                }
            }
            it = contents.next.clone();
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
