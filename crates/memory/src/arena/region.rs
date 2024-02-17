use arrayvec::ArrayVec;
use bootloader_api::info::{MemoryRegion, MemoryRegionKind, MemoryRegions};
use log::info;
use x86_64::PhysAddr;
use crate::arena::util::MergeMemoryRegions;

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
struct Region {
    ptr: PhysAddr,
    len: usize,
    used: usize,
}

impl Region {
    #[inline]
    pub fn new(m: &MemoryRegion) -> Self {
        Region {
            ptr: PhysAddr::new(m.start),
            len: (m.end - m.start) as usize,
            used: 0,
        }
    }

    #[inline]
    pub fn free(&self) -> usize {
        self.len - self.used
    }

    pub fn alloc(&mut self, size: usize) -> PhysAddr {
        if self.used + size > self.len {
            panic!("Cannot allocate {} bytes ({} already used): len {} exceeded", size, self.used, self.len);
        }
        let addr = self.ptr + self.used;
        self.used += size;
        addr
    }
}

pub struct RegionAllocator {
    regions: ArrayVec<Region, 32>,
}

impl RegionAllocator {
    pub fn new(regions: &MemoryRegions) -> Self {
        let regions: ArrayVec<Region, 32> = regions.iter()
            .filter(|r| r.kind == MemoryRegionKind::Usable)
            .copied()
            .merge_regions()
            .map(|r| Region::new(&r))
            .collect();
        info!("Region allocator initialized with regions {:?}", &regions);
        let total: usize = regions.iter().map(|r| r.free()).sum();
        info!("Total available memory: {} MiB", total / 1024 / 1024);
        RegionAllocator {
            regions
        }
    }

    pub fn allocate(&mut self, size: usize) -> Option<PhysAddr> {
        if size == 0 {
            panic!("Cannot allocate 0 bytes");
        }
        for reg in &mut self.regions {
            if reg.free() >= size {
                return Some(reg.alloc(size))
            }
        }
        None
    }
}