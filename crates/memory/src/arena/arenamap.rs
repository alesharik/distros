use crate::arena::{Arena, Error};
use core::mem::size_of;
use x86_64::{PhysAddr, VirtAddr};

#[derive(Eq, PartialEq, Copy, Clone)]
struct RawArena(Arena);

impl RawArena {
    pub fn is_empty(&self) -> bool {
        self.0.start.is_null()
    }

    pub fn is_taken(&self) -> bool {
        self.0.size >> 63 == 1
    }

    pub fn set_taken(&mut self, taken: bool) {
        if taken {
            self.0.size |= 1 << 63
        } else {
            self.0.size = self.0.size & (u64::MAX - 1 << 63)
        }
    }

    pub fn to_arena(&self) -> Arena {
        Arena {
            size: self.0.size & (u64::MAX - 1 << 63),
            start: self.0.start,
        }
    }

    #[inline]
    pub fn start(&self) -> PhysAddr {
        self.0.start
    }

    #[inline]
    pub fn size(&self) -> u64 {
        self.0.size
    }

    pub fn split(&mut self, size: usize) -> Arena {
        let arena = Arena {
            size: self.0.size - size as u64,
            start: self.0.start + size,
        };
        self.0.size = size as u64;
        arena
    }
}

pub struct ArenaMap {
    ptr: VirtAddr,
    len: usize,
    size: usize,
}

impl ArenaMap {
    pub fn new(ptr: VirtAddr, virt_size: usize) -> ArenaMap {
        ArenaMap {
            ptr,
            len: 0,
            size: virt_size / size_of::<RawArena>(),
        }
    }

    pub fn push(&mut self, arena: Arena, taken: bool) -> Result<(), Error> {
        if self.len == self.size {
            return Err(Error::ArenaMapSizeExhausted);
        }
        unsafe {
            let ptr = self.ptr + self.len * size_of::<RawArena>();
            let ptr: *mut RawArena = ptr.as_mut_ptr();
            *ptr = RawArena(arena);
            (&mut *ptr).set_taken(taken);
            self.len += 1;
        }
        Ok(())
    }

    fn try_push_at_pos(&mut self, pos: usize, arena: Arena) -> bool {
        if pos >= self.len {
            return false;
        }
        unsafe {
            let ptr = self.ptr + pos * size_of::<RawArena>();
            let ptr: *mut RawArena = ptr.as_mut_ptr();
            if (&*ptr).is_empty() {
                *ptr = RawArena(arena);
                return true;
            }
        }
        false
    }

    pub fn alloc(&mut self, size: usize) -> Result<Option<Arena>, Error> {
        if size == 0 {
            return Err(Error::SizeInvalid);
        }
        for i in 0..self.len {
            unsafe {
                let ptr = self.ptr + i * size_of::<RawArena>();
                let ptr: *mut RawArena = ptr.as_mut_ptr();
                if (&*ptr).is_taken() || (&*ptr).is_empty() {
                    continue;
                }
                if (&*ptr).size() == size as u64 {
                    (&mut *ptr).set_taken(true);
                    return Ok(Some((&*ptr).to_arena()));
                } else if (&*ptr).size() > size as u64 {
                    let new_arena = (&mut *ptr).split(size);
                    if !self.try_push_at_pos(i + 1, new_arena) {
                        self.push(new_arena, false)?;
                    }
                    return Ok(Some((&*ptr).to_arena()));
                }
            }
        }
        Ok(None)
    }

    pub fn dealloc(&mut self, start: PhysAddr) {
        if start.is_null() {
            return;
        }
        for i in 0..self.len {
            unsafe {
                let ptr = self.ptr + i * size_of::<RawArena>();
                let ptr: *mut RawArena = ptr.as_mut_ptr();
                if (&*ptr).start() == start {
                    (&mut *ptr).set_taken(false);
                    break;
                }
            }
        }
    }
}
