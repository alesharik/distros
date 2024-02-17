use spin::{Mutex, MutexGuard};
use x86_64::structures::paging::page::NotGiantPageSize;
use x86_64::structures::paging::{FrameAllocator, Mapper, OffsetPageTable, Page, PageSize, PageTable, PageTableFlags, PhysFrame, Size1GiB, Size2MiB, Size4KiB};
use x86_64::structures::paging::mapper::{MapToError, UnmapError};
use x86_64::VirtAddr;
use crate::frame_alloc::FrameAlloc;
use crate::translate_kernel;

static mut PAGE_TABLE: Option<Mutex<OffsetPageTable<'static>>> = None;

unsafe fn active_level_4_table() -> &'static mut PageTable {
    use x86_64::registers::control::Cr3;

    let (level_4_table_frame, _) = Cr3::read();

    let phys = level_4_table_frame.start_address();
    let virt = translate_kernel(phys);
    let page_table_ptr: *mut PageTable = virt.as_mut_ptr();

    &mut *page_table_ptr // unsafe
}

pub fn init(phys_offset: VirtAddr) {
    unsafe {
        PAGE_TABLE = Some(Mutex::new(OffsetPageTable::new(
            active_level_4_table(),
            phys_offset,
        )));
    }
}

fn get_table<'a>() -> MutexGuard<'a, OffsetPageTable<'static>> {
    unsafe { PAGE_TABLE.as_ref().expect("Page table not initialized").lock() }
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
    let mut table = get_table();
    unsafe {
        table.map_to(page, frame, flags, &mut FrameAlloc)
            .map(|f| f.flush())
    }
}

/// Unmap virtual page
pub fn unmap<T: NotGiantPageSize>(page: Page<T>) -> Result<PhysFrame<T>, UnmapError>
    where
            for<'a> OffsetPageTable<'a>: Mapper<T>,
{
    let mut table = get_table();
    table.unmap(page)
        .map(|(p, f)| {
            f.flush();
            p
        })
}