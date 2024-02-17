mod alloc;
mod arenamap;
mod region;
mod util;

pub use alloc::ArenaAllocator;

use crate::translate_kernel;
use bootloader_api::info::MemoryRegions;
use spin::{Mutex, MutexGuard};
use talc::Span;
use x86_64::structures::paging::{PageSize, PhysFrame};
use x86_64::PhysAddr;

const TWO_MIBS: usize = 2 * 1024 * 1024;
const ARENA_MAP_SIZE: usize = TWO_MIBS;

#[derive(Debug)]
pub enum Error {
    SizeInvalid,
    ArenaMapSizeExhausted,
    OutOfMemory,
}

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub struct Arena {
    start: PhysAddr,
    size: u64,
}

impl Arena {
    #[inline]
    pub fn start(&self) -> PhysAddr {
        self.start
    }

    #[inline]
    pub fn size(&self) -> u64 {
        self.size
    }

    #[inline]
    pub fn end(&self) -> PhysAddr {
        self.start + self.size
    }
}

impl<S: PageSize> Into<PhysFrame<S>> for Arena {
    fn into(self) -> PhysFrame<S> {
        PhysFrame::<S>::containing_address(self.start)
    }
}

impl Into<Span> for Arena {
    fn into(self) -> Span {
        let addr = translate_kernel(self.start);
        Span::from_base_size(addr.as_mut_ptr(), self.size as usize)
    }
}

static mut ARENA_ALLOC: Option<Mutex<ArenaAllocator>> = None;

pub fn initialize(regions: &MemoryRegions) {
    unsafe {
        ARENA_ALLOC = Some(Mutex::new(ArenaAllocator::new(regions)));
    }
}

pub fn arena_alloc<'a>() -> MutexGuard<'a, ArenaAllocator> {
    unsafe {
        ARENA_ALLOC
            .as_ref()
            .expect("Arena allocator not initialized")
            .lock()
    }
}
