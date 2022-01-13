//! Memory region manager
use arrayvec::ArrayVec;
use bootloader::bootinfo::{MemoryMap, MemoryRegion, MemoryRegionType};
use x86_64::PhysAddr;
use x86_64::structures::paging::{Size4KiB, PageSize};

/// This container holds memory region information and allows to take frames from it
struct MemoryRegionContainer {
    end: u64,
    pointer: u64,
}

impl MemoryRegionContainer {
    /// Create new container
    fn new(region: &MemoryRegion) -> Self {
        MemoryRegionContainer {
            end: region.range.end_frame_number,
            pointer: region.range.start_frame_number,
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
    pub fn new(map: &'static MemoryMap, offsets: &[u64]) -> MemoryRegionProvider {
        let mut regions = ArrayVec::<MemoryRegionContainer, 16>::new();
        for region in map
            .iter()
            .filter(|m| m.region_type == MemoryRegionType::Usable)
        {
            regions.push(MemoryRegionContainer::new(region));
        }
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