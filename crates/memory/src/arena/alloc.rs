use bootloader_api::info::MemoryRegions;
use log::info;
use x86_64::PhysAddr;
use crate::arena::{Arena, ARENA_MAP_SIZE, Error, TWO_MIBS};
use crate::arena::arenamap::ArenaMap;
use crate::arena::region::RegionAllocator;
use crate::translate_kernel;

pub struct ArenaAllocator {
    region: RegionAllocator,
    map: ArenaMap,
}

impl ArenaAllocator {
    pub fn new(regions: &MemoryRegions) -> Self {
        let mut region = RegionAllocator::new(regions);
        let map_virt = translate_kernel(
            region.allocate(ARENA_MAP_SIZE)
                .expect("Failed to allocate arena map")
        );
        info!("Created arena map at {:?}", map_virt);

        ArenaAllocator {
            region,
            map: ArenaMap::new(map_virt, ARENA_MAP_SIZE),
        }
    }

    /// Allocates minimum 2-mibs spaces
    pub fn allocate(&mut self, size: usize) -> Result<Arena, Error> {
        if size == 0 {
            return Err(Error::SizeInvalid);
        }
        let size = if size % TWO_MIBS == 0 {
            size
        } else {
            (size / TWO_MIBS + 1) * TWO_MIBS
        };
        match self.map.alloc(size)? {
            Some(arena) => Ok(arena),
            None => match self.region.allocate(size) {
                None => Err(Error::OutOfMemory),
                Some(addr) => {
                    let arena = Arena { start: addr, size: size as u64 };
                    self.map.push(arena, true)?;
                    Ok(arena)
                }
            }
        }
    }

    pub fn deallocate(&mut self, start: PhysAddr) {
        self.map.dealloc(start)
    }
}