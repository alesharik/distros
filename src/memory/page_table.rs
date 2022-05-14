//! This module manages all page table stuff

use super::frame;
use spin::Mutex;
use x86_64::structures::paging::mapper::{MapToError, TranslateResult, UnmapError};
use x86_64::structures::paging::page::NotGiantPageSize;
use x86_64::structures::paging::page_table::PageTableEntry;
use x86_64::structures::paging::{
    FrameAllocator, Mapper, OffsetPageTable, Page, PageTable, PageTableFlags, PhysFrame, Size4KiB,
    Translate,
};
use x86_64::{PhysAddr, VirtAddr};

lazy_static! {
    static ref PAGE_TABLE: Mutex<Option<OffsetPageTable<'static>>> = Mutex::new(None);
}

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

/// Map virtual page
pub fn map<T: NotGiantPageSize>(
    frame: PhysFrame<T>,
    page: Page<T>,
    flags: PageTableFlags,
) -> Result<(), MapToError<T>>
where
    for<'a> OffsetPageTable<'a>: Mapper<T>,
{
    frame::with_frame_alloc(|a| {
        match unsafe {
            let mut table = PAGE_TABLE.lock();
            let x = table.as_mut().unwrap();
            x.map_to(page, frame, flags, a)
        } {
            Ok(f) => {
                f.flush();
                Ok(())
            }
            Err(e) => Err(e),
        }
    })
}

/// Unmap virtual page
pub fn unmap<T: NotGiantPageSize>(page: Page<T>) -> Result<PhysFrame<T>, UnmapError>
where
    for<'a> OffsetPageTable<'a>: Mapper<T>,
{
    match unsafe {
        let mut table = PAGE_TABLE.lock();
        let x = table.as_mut().unwrap();
        x.unmap(page)
    } {
        Ok((frame, f)) => {
            f.flush();
            Ok(frame)
        }
        Err(e) => Err(e),
    }
}

pub fn map_init<T: NotGiantPageSize, A: FrameAllocator<Size4KiB> + Sized>(
    frame: PhysFrame<T>,
    page: Page<T>,
    flags: PageTableFlags,
    allocator: &mut A,
) -> Result<(), MapToError<T>>
where
    for<'a> OffsetPageTable<'a>: Mapper<T>,
{
    match unsafe {
        let mut table = PAGE_TABLE.lock();
        table
            .as_mut()
            .unwrap()
            .map_to(page, frame, flags, allocator)
    } {
        Ok(f) => {
            f.flush();
            Ok(())
        }
        Err(e) => Err(e),
    }
}

/// Translate virtual address to physical address
///
/// # Return
/// Physical address if have one, otherwise None
pub fn translate(addr: VirtAddr) -> Option<PhysAddr> {
    match unsafe {
        let mut table = PAGE_TABLE.lock();
        table.as_ref().unwrap().translate(addr)
    } {
        TranslateResult::Mapped {
            frame,
            offset,
            flags: _flags,
        } => Some(frame.start_address() + offset),
        _ => None,
    }
}

pub struct P3PageTable {
    entry: PageTableEntry,
    index: u16,
}

impl P3PageTable {
    pub fn take(index: usize) -> P3PageTable {
        if index > 512 {
            panic!("Invalid P3 page table index - {}", index);
        }
        let mut table = PAGE_TABLE.lock();
        let mut p4 = table.as_mut().unwrap().level_4_table();
        let table = p4[index].clone();
        p4[index].set_unused();
        P3PageTable {
            entry: table,
            index: index as u16,
        }
    }

    pub fn restore(self) {
        let mut table = PAGE_TABLE.lock();
        let mut p4 = table.as_mut().unwrap().level_4_table();
        p4[self.index as usize] = self.entry
    }
}
