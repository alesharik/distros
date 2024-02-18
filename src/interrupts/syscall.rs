use crate::interrupts;
use crate::interrupts::InterruptId;
use distros_interrupt::OverrideMode;
use libkernel::syscall::{
    self, take_command, SyscallCommand, SyscallMessage, SYSCALL_IN_MEM, SYSCALL_SYNC_MEM,
};
use x86_64::structures::idt::InterruptStackFrame;
use x86_64::structures::paging::PageTableFlags;
use x86_64::VirtAddr;

int_handler!(noint syscall_handler | _stack_frame: InterruptStackFrame | {
    if let Some(cmd) = take_command() {
        match cmd {
            SyscallCommand::Test => {
                debug!("SYSCALL: TEST");
                unsafe {
                    syscall::call_handler(SyscallMessage::Test);
                }
            }
            SyscallCommand::Test1 => {
                debug!("SYSCALL: TEST1");
            }
        }
    }
});

/// Setup global syscall handlers
pub fn init() {
    distros_interrupt::set_handler(InterruptId::new(80), syscall_handler, OverrideMode::Panic);
}

/// Setup syscall memory for program
pub fn init_syscall_block() -> () {
    // memory::util::static_map_memory(
    //     VirtAddr::new_truncate(SYSCALL_IN_MEM),
    //     4096,
    //     PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::USER_ACCESSIBLE,
    // )?;
    // memory::util::static_map_memory(
    //     VirtAddr::new_truncate(SYSCALL_SYNC_MEM),
    //     4096,
    //     PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::USER_ACCESSIBLE,
    // )
}
