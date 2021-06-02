use spin::{Mutex, Lazy};
use x86_64::{
    PhysAddr,
    structures::paging::{Page, PageTable},
    VirtAddr
};
use x86_64::structures::paging::{FrameAllocator, Mapper, OffsetPageTable, PhysFrame, Size2MiB, Size4KiB};
use x86_64::structures::paging::mapper::{MapToError, TranslateError, UnmapError};
use x86_64::structures::paging::page::NotGiantPageSize;
use bootloader::bootinfo::{MemoryMap, MemoryRegionType, MemoryRegion};
use core::iter::{Map, FlatMap, Filter, StepBy};
use core::slice::Iter;
use core::ops::Range;

use crate::kblog;

mod page;
mod kheap;
mod liballoc;
mod process;

pub use kheap::init_kheap;
use crate::memory::page::FrameAlloc;


struct KFrameAlloc<'a> {
    iterator: Map<FlatMap<Map<Filter<Iter<'a, MemoryRegion>, fn(&&MemoryRegion) -> bool>, fn(&MemoryRegion) -> Range<u64>>, StepBy<Range<u64>>, fn(Range<u64>) -> StepBy<Range<u64>>>, fn(u64) -> PhysFrame<Size4KiB>>,
    hugepage_iterator: Map<FlatMap<Map<Filter<Iter<'a, MemoryRegion>, fn(&&MemoryRegion) -> bool>, fn(&MemoryRegion) -> Range<u64>>, StepBy<Range<u64>>, fn(Range<u64>) -> StepBy<Range<u64>>>, fn(u64) -> PhysFrame<Size2MiB>>
}

impl<'a> KFrameAlloc<'a> {
    fn new(memory_map: &'static MemoryMap) -> KFrameAlloc<'a> {
        let iter: Iter<'a, MemoryRegion> = memory_map.iter();
        let flt: fn(&&MemoryRegion) -> bool = |r| r.region_type == MemoryRegionType::Usable;
        let map1: fn(&MemoryRegion) -> Range<u64> = |r: &MemoryRegion| r.range.start_addr()..r.range.end_addr();
        let flatmap: fn(Range<u64>) -> StepBy<Range<u64>> = |r: Range<u64>| r.step_by(4096);
        let flatmap_huge: fn(Range<u64>) -> StepBy<Range<u64>> = |r: Range<u64>| r.step_by(2 * 1024 * 1024);
        let map2: fn(u64) -> PhysFrame<Size4KiB> = |addr: u64| PhysFrame::containing_address(PhysAddr::new(addr));
        let map2_huge: fn(u64) -> PhysFrame<Size2MiB> = |addr: u64| PhysFrame::containing_address(PhysAddr::new(addr));
        let regions = iter
            .filter(flt)
            .map(map1)
            .flat_map(flatmap)
            .map(map2);
        let iter1: Iter<'a, MemoryRegion> = memory_map.iter();
        let regions_huge = iter1
            .filter(flt)
            .map(map1)
            .flat_map(flatmap_huge)
            .map(map2_huge);
        KFrameAlloc {
            iterator: regions,
            hugepage_iterator: regions_huge
        }
    }
}

unsafe impl FrameAllocator<Size4KiB> for KFrameAlloc<'_> {
    fn allocate_frame(&mut self) -> Option<PhysFrame<Size4KiB>> {
        self.iterator.next()
    }
}

unsafe impl FrameAllocator<Size2MiB> for KFrameAlloc<'_> {
    fn allocate_frame(&mut self) -> Option<PhysFrame<Size2MiB>> {
        self.hugepage_iterator.next()
    }
}

#[derive(Debug)]
pub enum DeallocatingError {
    TranslateError(TranslateError),
    UnmapError(UnmapError),
}

pub struct MemoryManager {
}

impl MemoryManager {
    // fn allocate_frames(start: Page<Size4KiB>, frames: u32) -> Result<(), MapToError<Size4KiB>> {
    //     use x86_64::structures::paging::PageTableFlags as Flags;
    //     unsafe {
    //         let mut guard = FRAME_ALLOCATOR.lock();
    //         let frame_allocator = guard.as_mut().expect("Frame allocator is not setup!");
    //         let frame: PhysFrame<Size4KiB> = frame_allocator.allocate(frames)
    //             .ok_or(MapToError::FrameAllocationFailed)?;
    //         let flags = Flags::PRESENT | Flags::WRITABLE;
    //         let mut table = PAGE_TABLE.lock();
    //         table.as_mut().expect("Page table is not setup!").map_to(start, frame, flags, frame_allocator)?.flush()
    //     }
    //     Ok(())
    // }

    // fn deallocate_frames(start: Page<Size4KiB>) -> Result<(), DeallocatingError> {
    //     unsafe {
    //         let mut table = PAGE_TABLE.lock();
    //         let mut t = table.as_mut().expect("Page table is not setup!");
    //         let phys = t.translate_page(start).map_err(|e| DeallocatingError::TranslateError(e))?;
    //         t.unmap(start).map_err(|e| DeallocatingError::UnmapError(e))?;
    //         let mut guard = FRAME_ALLOCATOR.lock();
    //         let frame_allocator = guard.as_mut().expect("Frame allocator is not setup!");
    //         frame_allocator.deallocate(phys);
    //     }
    //     Ok(())
    // }
}

pub trait AllocatePage<T: NotGiantPageSize = Size4KiB> {
    fn allocate(page: Page<T>) -> Result<(), MapToError<T>>;
}

lazy_static!(
    static ref PAGE_TABLE: Mutex<Option<OffsetPageTable<'static>>> = Mutex::new(Option::None);
    static ref FRAME_ALLOCATOR: Mutex<Option<KFrameAlloc<'static>>> = Mutex::new(Option::None);
    static ref PHYS_OFFSET: Mutex<Option<VirtAddr>> = Mutex::new(Option::None);
);

unsafe fn active_level_4_table(phys_offset: VirtAddr) -> &'static mut PageTable {
    use x86_64::registers::control::Cr3;

    let (level_4_table_frame, _) = Cr3::read();

    let phys = level_4_table_frame.start_address();
    let virt = phys_offset + phys.as_u64();
    let page_table_ptr: *mut PageTable = virt.as_mut_ptr();

    &mut *page_table_ptr // unsafe
}

pub fn init_memory(phys_offset: VirtAddr, memory_map: &'static MemoryMap) {
    unsafe {
        let mut page_table = PAGE_TABLE.lock();
        *page_table = Option::Some(OffsetPageTable::new(active_level_4_table(phys_offset), phys_offset));
        let mut frame_allocator = FRAME_ALLOCATOR.lock();
        *frame_allocator = Option::Some(KFrameAlloc::new(memory_map));
        let mut phys_off = PHYS_OFFSET.lock();
        *phys_off = Some(phys_offset.clone());
    }
    kblog!("MemoryManager", "Memory manager ready");
}

pub fn print_table() {
    let mut guard = PAGE_TABLE.lock();
    let l4_table = guard.as_mut().expect("Page table is not setup!").level_4_table();
    for (i, entry) in l4_table.iter().enumerate() {
        if !entry.is_unused() {
            kblog!("MemoryManager", "L4 Entry {}: {:?}", i, entry);
        }
    }
}

pub fn map_physical_address(address: PhysAddr) -> VirtAddr {
    let off = PHYS_OFFSET.lock();
    VirtAddr::new(address.as_u64() + off.as_ref().expect("Physical offset is not setup!").as_u64())
}

impl AllocatePage<Size4KiB> for MemoryManager {
    fn allocate(page: Page<Size4KiB>) -> Result<(), MapToError<Size4KiB>> {
        use x86_64::structures::paging::PageTableFlags as Flags;
        unsafe {
            let mut guard = FRAME_ALLOCATOR.lock();
            let frame_allocator = guard.as_mut().expect("Frame allocator is not setup!");
            let frame: PhysFrame<Size4KiB> = frame_allocator.allocate_frame()
                .ok_or(MapToError::FrameAllocationFailed)?;
            let flags = Flags::PRESENT | Flags::WRITABLE;
            let mut table = PAGE_TABLE.lock();
            table.as_mut().expect("Page table is not setup!").map_to(page, frame, flags, frame_allocator)?.flush()
        }
        Ok(())
    }
}

impl AllocatePage<Size2MiB> for MemoryManager {
    fn allocate(page: Page<Size2MiB>) -> Result<(), MapToError<Size2MiB>> {
        use x86_64::structures::paging::PageTableFlags as Flags;
        unsafe {
            let mut guard = FRAME_ALLOCATOR.lock();
            let frame_allocator = guard.as_mut().expect("Frame allocator is not setup!");
            let frame: PhysFrame<Size2MiB> = frame_allocator.allocate_frame()
                .ok_or(MapToError::FrameAllocationFailed)?;
            let flags = Flags::PRESENT | Flags::WRITABLE;
            let mut table = PAGE_TABLE.lock();
            table.as_mut().expect("Page table is not setup!").map_to(page, frame, flags, frame_allocator)?.flush()
        }
        Ok(())
    }
}
