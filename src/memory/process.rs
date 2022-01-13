use x86_64::structures::paging::PageTable;
use alloc::vec::Vec;
use crate::memory::util::MemoryToken;
use crate::memory::liballoc::process_heap_inner;
use x86_64::VirtAddr;
use itertools::Itertools;
use crate::memory::page_table::P3PageTable;

const PROCESS_HEAP_START: u64 = 0x_6000_0000_0000;

struct PageTableRange {
    start: u64,
    end: u64,
}

impl PageTableRange {
    fn size(&self) -> usize {
        (self.end - self.start) as usize
    }
}

struct PageTableInfo {
    p4_index: usize,
    free_ranges: Vec<PageTableRange>,
    busy_ranges: Vec<PageTableRange>,
}

impl PageTableInfo {
    fn new(index: usize) -> PageTableInfo {
        PageTableInfo {
            p4_index: index,
            free_ranges: vec![PageTableRange {
                start: (index * 1024 * 1024 * 1024) as u64,
                end: ((index + 1) * 1024 * 1024 * 1024) as u64
            }],
            busy_ranges: vec![],
        }
    }

    fn try_allocate(&mut self, size: usize) -> Option<VirtAddr> {
        let idx = {
            let mut idx = -1isize;
            for (i, range) in self.free_ranges.iter().enumerate() {
                if range.size() >= size {
                    idx = i as isize;
                }
            }
            if idx == -1 {
                return None
            }
            idx
        } as usize;
        let mut range = self.free_ranges.remove(idx);
        let taken_range_start = range.start;
        let taken_range = PageTableRange {
            start: taken_range_start,
            end: (range.start + size as u64),
        };
        self.busy_ranges.push(taken_range);
        range.start += size as u64;
        if range.size() > 0 {
            self.free_ranges.push(range);
        }
        Some(VirtAddr::new(taken_range_start))
    }

    fn deallocate(&mut self, addr: VirtAddr, size: usize) {
        if let Some((idx, _)) = self.busy_ranges.iter().find_position(|e| e.start == addr.as_u64() && e.size() == size) {
            let my_range = self.busy_ranges.remove(idx);
            self.free_ranges.push(my_range);
            for idx in (1..self.free_ranges.len()).rev() {
                if self.free_ranges[idx - 1].end == self.free_ranges[idx].start {
                    self.free_ranges[idx - 1].end += self.free_ranges[idx].size() as u64;
                    self.free_ranges.remove(idx);
                } else {
                    break
                }
            }
        }
    }
}

pub struct ProcessMemoryBackup {
    info: ProcessMappingInfo,
    pages: Vec<P3PageTable>,
}

impl ProcessMemoryBackup {
    pub fn restore(self) -> ProcessMappingInfo {
        for table in self.pages.into_iter() {
            table.restore();
        }
        self.info
    }
}

pub struct ProcessMappingInfo {
    tables: Vec<PageTableInfo>,
}

impl ProcessMappingInfo {
    pub fn new() -> ProcessMappingInfo {
        ProcessMappingInfo {
            tables: vec![],
        }
    }

    pub fn backup(self) -> ProcessMemoryBackup {
        let pages = self.tables.iter().map(|p| P3PageTable::take(p.p4_index)).collect::<Vec<_>>();
        ProcessMemoryBackup {
            info: self,
            pages
        }
    }

    pub fn allocate_virt(&mut self, size: usize) -> VirtAddr {
        if size > 1024 * 1024 * 1024 {
            panic!("Does not support page allocation with size > 1GiB")
        }
        for x in self.tables.iter_mut() {
            if let Some(addr) = x.try_allocate(size) {
                return addr
            }
        }
        let new_page = PROCESS_HEAP_START + (self.tables.len() * 1024 * 1024 * 1024) as u64;
        self.tables.push(PageTableInfo::new((new_page / 1024 / 1024 / 1024) as usize));
        VirtAddr::new(new_page)
    }

    pub fn deallocate_virt(&mut self, addr: VirtAddr, size: usize) {
        let page = addr.as_u64() / 1024 / 1024 / 1024;
        let page = self.tables.iter_mut().find(|p| p.p4_index == page as usize);
        if let Some(mut page) = page {
            page.deallocate(addr, size);
        }
    }
}