use alloc::vec::Vec;
use fixedbitset::FixedBitSet;
use hashbrown::{HashMap, HashSet};
use x86_64::{PhysAddr, VirtAddr};
use x86_64::structures::paging::{Page, Size4KiB, PageSize, PhysFrame, PageTableFlags};
use crate::memory::frame::with_frame_alloc;
use crate::memory::page_table::{map, P3PageTable, translate, unmap};

const PROCESS_HEAP_START: u64 = 0x_6000_0000_0000;

pub struct PageAllocator {
    frames: HashMap<u64, u64>,
    tables: FixedBitSet
}

impl PageAllocator {
    pub fn new() -> PageAllocator {
        PageAllocator {
            frames: HashMap::new(),
            tables: FixedBitSet::with_capacity(512)
        }
    }

    pub fn allocate(&mut self, pages: u64) -> Option<VirtAddr> {
        let frame = with_frame_alloc(|falloc| falloc.allocate(pages))?;
        let start_page = Page::<Size4KiB>::containing_address(
            VirtAddr::new(frame.start_address().as_u64() + PROCESS_HEAP_START)
        );
        let end_page = Page::<Size4KiB>::containing_address(
            VirtAddr::new(frame.start_address().as_u64() + PROCESS_HEAP_START + pages * Size4KiB::SIZE)
        );
        for (idx, p) in Page::range_inclusive(start_page, end_page).enumerate() {
            let phys = unsafe {
                PhysFrame::<Size4KiB>::from_start_address_unchecked(
                    PhysAddr::new_truncate(frame.start_address().as_u64() + idx as u64 * Size4KiB::SIZE)
                )
            };
            map(phys, p, PageTableFlags::PRESENT | PageTableFlags::WRITABLE).unwrap(); // fixme
            self.tables.set(usize::from(p.p4_index()), true);
        }
        self.frames.insert(frame.start_address().as_u64(), pages);
        Some(start_page.start_address())
    }

    pub fn deallocate(&mut self, addr: VirtAddr) {
        let frame = match translate(addr) {
            Some(r) => r,
            None => return
        };
        if !self.frames.contains_key(&frame.as_u64()) {
            return;
        }
        let pages = self.frames[&frame.as_u64()];
        let start_page = Page::<Size4KiB>::containing_address(
            VirtAddr::new(frame.as_u64() + PROCESS_HEAP_START)
        );
        let end_page = Page::<Size4KiB>::containing_address(
            VirtAddr::new(frame.as_u64() + PROCESS_HEAP_START + pages * Size4KiB::SIZE)
        );
        for page in Page::range_inclusive(start_page, end_page) {
            unmap(page);
        }
        with_frame_alloc(|falloc| falloc.deallocate(PhysFrame::containing_address(frame)));
        self.frames.remove(&frame.as_u64());
    }

    pub fn backup(self) -> PageAllocatorBackup {
        let mut tables = Vec::new();
        for i in 0..512 {
            if self.tables[i] {
                tables.push(P3PageTable::take(i));
            }
        }
        PageAllocatorBackup {
            allocator: self,
            tables
        }
    }
}

impl Drop for PageAllocator {
    fn drop(&mut self) {
        with_frame_alloc(|falloc| {
            for addr in self.frames.keys() {
                falloc.deallocate(PhysFrame::containing_address(PhysAddr::new(*addr)));
            }
        });
    }
}

pub struct PageAllocatorBackup {
    allocator: PageAllocator,
    tables: Vec<P3PageTable>
}

impl PageAllocatorBackup {
    pub fn restore(self) -> PageAllocator {
        for x in self.tables {
            x.restore();
        }
        self.allocator
    }
}