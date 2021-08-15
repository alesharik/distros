use crate::interrupts;
use crate::memory::util::{MemoryError, MemoryToken};
use alloc::vec::Vec;
use goblin::elf::program_header::{PT_GNU_STACK, PT_LOAD};
use goblin::elf::Elf;
use goblin::error::Error;
use x86_64::structures::paging::PageTableFlags;
use x86_64::VirtAddr;

#[derive(Debug)]
pub enum ElfError {
    Elf(Error),
    Memory(MemoryError),
}

pub struct ElfProgram {
    tokens: Vec<MemoryToken>,
    entry: u64,
}

impl ElfProgram {
    pub fn load(data: &[u8]) -> Result<ElfProgram, ElfError> {
        let elf = Elf::parse(data).map_err(|e| ElfError::Elf(e))?;
        let mut tokens = Vec::with_capacity(elf.program_headers.len());
        for x in elf.program_headers {
            match x.p_type {
                PT_LOAD => {
                    let range = x.vm_range();
                    let vm_size = range.end - range.start;
                    let file_range = x.file_range();
                    let file_size = file_range.end - file_range.start;
                    let mut flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
                    if !x.is_executable() {
                        flags |= PageTableFlags::NO_EXECUTE;
                    }
                    let token = crate::memory::util::static_map_memory(
                        VirtAddr::new_truncate(range.start as u64),
                        file_size,
                        flags,
                    )
                    .map_err(|e| ElfError::Memory(e))?;
                    unsafe {
                        core::ptr::copy(
                            (data.as_ptr() as usize + file_range.start) as *const u8,
                            range.start as *mut u8,
                            file_size,
                        );
                        if vm_size > file_size {
                            core::ptr::write_bytes(
                                (range.start + file_size) as *mut u8,
                                0,
                                vm_size - file_size,
                            );
                        }
                    }
                    tokens.push(token);
                }
                PT_GNU_STACK => {
                    let range = x.vm_range();
                    let token = crate::memory::util::static_map_memory(
                        VirtAddr::new_truncate(range.start as u64),
                        range.end - range.start,
                        PageTableFlags::PRESENT
                            | PageTableFlags::WRITABLE
                            | PageTableFlags::NO_EXECUTE,
                    )
                    .map_err(|e| ElfError::Memory(e))?;
                    unsafe {
                        core::ptr::write_bytes(range.start as *mut u8, 0, range.end - range.start);
                    }
                    tokens.push(token);
                }
                _ => {}
            }
        }
        let syscall_token = interrupts::init_syscall_block().map_err(|e| ElfError::Memory(e))?;
        tokens.push(syscall_token);
        Ok(ElfProgram {
            tokens,
            entry: elf.entry,
        })
    }

    pub fn start_tmp(&self) {
        unsafe {
            asm!("jmp {}", in(reg) self.entry);
        }
    }
}
