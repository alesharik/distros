//! Memory region manager
use arrayvec::ArrayVec;
use bootloader_api::info::{MemoryRegion, MemoryRegionKind, MemoryRegions};
use x86_64::structures::paging::{PageSize, Size4KiB};
use x86_64::PhysAddr;
use crate::memory::util::MergeMemoryRegions;

/// This container holds memory region information and allows to take frames from it
#[derive(Debug)]
struct MemoryRegionContainer {
    pointer: u64,
    end: u64,
}

impl MemoryRegionContainer {
    /// Create new container
    fn new(region: MemoryRegion) -> Self {
        MemoryRegionContainer {
            end: region.end,
            pointer: region.start,
        }
    }

    /// Take frames from this container
    ///
    /// # Returns
    /// Taken frame or `None` if container does not have enough space to reserve requested frames
    fn take(&mut self, frames: u64) -> Option<PhysAddr> {
        if self.end - self.pointer >= frames {
            let frame = PhysAddr::new(self.pointer * Size4KiB::SIZE);
            self.pointer += frames;
            Some(frame)
        } else {
            None
        }
    }
}

/// This struct provides unused frames from memory regions
pub struct MemoryRegionProvider {
    regions: ArrayVec<MemoryRegionContainer, 16>,
}

impl MemoryRegionProvider {
    /// Create new provider
    ///
    /// # Arguments
    /// - `map` - global memory map
    /// - `offsets` - how much memory is used from every usable memory region
    pub fn new(map: &'static MemoryRegions, offsets: &[u64]) -> MemoryRegionProvider {
        let mut regions: ArrayVec<_, 16> = map
            .iter()
            .filter(|m| m.kind == MemoryRegionKind::Usable)
            .copied()
            .merge_regions()
            .map(|e| MemoryRegionContainer::new(e))
            .collect();

        for (i, x) in regions.iter_mut().enumerate() {
            if let Some(off) = offsets.get(i) {
                let mut page_off = *off / Size4KiB::SIZE;
                if page_off * Size4KiB::SIZE != *off {
                    page_off += 1;
                }
                x.pointer += page_off + 1;
            }
        }
        MemoryRegionProvider { regions }
    }

    /// Take and reserve frames from memory
    ///
    /// # Returns
    /// Start address or `None` if no memory left
    pub fn take(&mut self, frames: u64) -> Option<PhysAddr> {
        for reg in &mut self.regions {
            if let Some(phys) = reg.take(frames as u64) {
                return Some(phys);
            }
        }
        None
    }
}
