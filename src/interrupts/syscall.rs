use crate::interrupts;
use crate::interrupts::InterruptId;
use crate::memory;
use x86_64::structures::idt::InterruptStackFrame;
use x86_64::VirtAddr;
use x86_64::structures::paging::PageTableFlags;
use crate::memory::util::{MemoryToken, MemoryError};
use libkernel::syscall::{take_command, SYSCALL_IN_MEM, SYSCALL_SYNC_MEM, SyscallCommand, self, SyscallMessage};

int_handler!(noint syscall_handler | stack_frame: InterruptStackFrame | {
    if let Some(cmd) = take_command() {
        match cmd {
            SyscallCommand::Test => {
                kblog!("SYSCALL", "TEST");
                unsafe {
                    syscall::call_handler(SyscallMessage::Test);
                }
            }
            SyscallCommand::Test1 => {
                kblog!("SYSCALL", "TEST1");
            }
        }
    }
});

/// Setup global syscall handlers
pub fn init() {
    interrupts::set_handler(InterruptId::from_raw(80), syscall_handler);
}

/// Setup syscall memory for program
pub fn init_syscall_block() -> Result<MemoryToken, MemoryError> {
    memory::util::static_map_memory(
        VirtAddr::new_truncate(SYSCALL_IN_MEM),
        4096,
        PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::USER_ACCESSIBLE
    )?;
    memory::util::static_map_memory(
        VirtAddr::new_truncate(SYSCALL_SYNC_MEM),
        4096,
        PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::USER_ACCESSIBLE
    )
}