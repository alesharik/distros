//! This module manages all page table stuff

use spin::Mutex;
use x86_64::structures::paging::{OffsetPageTable, PageTable, Mapper, PhysFrame, Size4KiB, Page, PageTableFlags, FrameAllocator};
use x86_64::VirtAddr;
use super::frame;
use x86_64::structures::paging::mapper::MapToError;
use x86_64::structures::paging::page::NotGiantPageSize;

lazy_static!(
    static ref PAGE_TABLE: Mutex<Option<OffsetPageTable<'static>>> = Mutex::new(None);
);

unsafe fn active_level_4_table(phys_offset: VirtAddr) -> &'static mut PageTable {
    use x86_64::registers::control::Cr3;

    let (level_4_table_frame, _) = Cr3::read();

    let phys = level_4_table_frame.start_address();
    let virt = phys_offset + phys.as_u64();
    let page_table_ptr: *mut PageTable = virt.as_mut_ptr();

    &mut *page_table_ptr // unsafe
}

pub fn init(phys_offset: VirtAddr) {
    unsafe {
        let mut page_table = PAGE_TABLE.lock();
        *page_table = Option::Some(OffsetPageTable::new(
            active_level_4_table(phys_offset),
            phys_offset,
        ));
    }
}

pub fn map<T: NotGiantPageSize>(frame: PhysFrame<T>, page: Page<T>, flags: PageTableFlags) -> Result<(), MapToError<T>>
    where for<'a> OffsetPageTable<'a>: Mapper<T> {
    frame::with_frame_alloc(|a| {
        match unsafe {
            let mut table = PAGE_TABLE.lock();
            let x = table.as_mut().unwrap();
            x.map_to(page, frame, flags, a)
        } {
            Ok(f) => {
                f.flush();
                Ok(())
            },
            Err(e) => Err(e)
        }
    })
}

pub fn map_init<T: NotGiantPageSize, A: FrameAllocator<Size4KiB> + Sized>(frame: PhysFrame<T>, page: Page<T>, flags: PageTableFlags, allocator: &mut A) -> Result<(), MapToError<T>>
    where for<'a> OffsetPageTable<'a>: Mapper<T> {
    match unsafe {
        let mut table = PAGE_TABLE.lock();
        table.as_mut().unwrap().map_to(page, frame, flags, allocator)
    } {
        Ok(f) => {
            f.flush();
            Ok(())
        },
        Err(e) => Err(e)
    }
}