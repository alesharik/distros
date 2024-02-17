use x86_64::structures::paging::{FrameAllocator, PageSize, PhysFrame, Size1GiB, Size2MiB, Size4KiB};
use crate::arena::arena_alloc;

pub static FRAME_ALLOC: FrameAlloc = FrameAlloc;

pub struct FrameAlloc;

unsafe impl FrameAllocator<Size4KiB> for FrameAlloc {
    fn allocate_frame(&mut self) -> Option<PhysFrame<Size4KiB>> {
        let mut arena = arena_alloc();
        arena
            .allocate(Size4KiB::SIZE as usize)
            .ok()
            .map(|a| a.into())
    }
}

unsafe impl FrameAllocator<Size2MiB> for FrameAlloc {
    fn allocate_frame(&mut self) -> Option<PhysFrame<Size2MiB>> {
        let mut arena = arena_alloc();
        arena
            .allocate(Size2MiB::SIZE as usize)
            .ok()
            .map(|a| a.into())
    }
}

unsafe impl FrameAllocator<Size1GiB> for FrameAlloc {
    fn allocate_frame(&mut self) -> Option<PhysFrame<Size1GiB>> {
        let mut arena = arena_alloc();
        arena
            .allocate(Size1GiB::SIZE as usize)
            .ok()
            .map(|a| a.into())
    }
}
