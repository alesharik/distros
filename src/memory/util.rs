use super::frame;
use super::page_table;
use crate::memory::AllocatePage;
use core::fmt::{Debug, Formatter};
use core::ops::Add;
use x86_64::structures::paging::mapper::MapToError;
use x86_64::structures::paging::{
    FrameAllocator, Page, PageTableFlags, PhysFrame, Size2MiB, Size4KiB,
};
use x86_64::VirtAddr;

pub enum MemoryError {
    NotEnoughMemory,
    PageTableError(MapToError<Size4KiB>),
}

impl Debug for MemoryError {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            MemoryError::NotEnoughMemory => write!(f, "Not enough memory"),
            MemoryError::PageTableError(e) => write!(f, "Page table error: {:?}", e),
        }
    }
}

pub struct MemoryToken {
    addr: PhysFrame<Size4KiB>,
}

impl Drop for MemoryToken {
    fn drop(&mut self) {
        let frame1 = self.addr;
        frame::with_frame_alloc(|a| a.deallocate(frame1))
    }
}

pub fn static_map_memory(
    address: VirtAddr,
    size: usize,
    flags: PageTableFlags,
) -> Result<MemoryToken, MemoryError> {
    let mut frames = size / 4096;
    if frames * 4096 < size {
        frames += 1;
    }
    let start_phys = match frame::with_frame_alloc(|a| a.allocate(frames as u64)) {
        Some(p) => p,
        None => return Err(MemoryError::NotEnoughMemory),
    };
    for frame in 0..frames {
        let phys_frame = start_phys.add(frame as u64);
        let virt_target = address.add(frame * 4096);
        page_table::map(phys_frame, Page::containing_address(virt_target), flags)
            .map_err(|e| MemoryError::PageTableError(e))?;
    }
    Ok(MemoryToken { addr: start_phys })
}

pub struct MemoryAllocator {}

impl AllocatePage<Size4KiB> for MemoryAllocator {
    fn allocate(page: Page<Size4KiB>) -> Result<(), MapToError<Size4KiB>> {
        frame::with_frame_alloc(|a| {
            let frame: PhysFrame<Size4KiB> = a
                .allocate_frame()
                .ok_or(MapToError::FrameAllocationFailed)?;
            page_table::map(
                frame,
                page,
                PageTableFlags::PRESENT | PageTableFlags::WRITABLE,
            )
        })?;
        Ok(())
    }
}

impl AllocatePage<Size2MiB> for MemoryAllocator {
    fn allocate(page: Page<Size2MiB>) -> Result<(), MapToError<Size2MiB>> {
        frame::with_frame_alloc(|a| {
            let frame: PhysFrame<Size2MiB> = a
                .allocate_frame()
                .ok_or(MapToError::FrameAllocationFailed)?;
            page_table::map(
                frame,
                page,
                PageTableFlags::PRESENT | PageTableFlags::WRITABLE,
            )
        })?;
        Ok(())
    }
}
