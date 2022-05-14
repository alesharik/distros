use core::arch::asm;

pub const SYSCALL_IN_MEM: u64 = 0x_5444_4444_0000;
pub const SYSCALL_SYNC_MEM: u64 = 0x_5444_4445_0000;

#[derive(Clone)]
pub enum SyscallCommand {
    Test,
    Test1,
}

pub fn take_command() -> Option<SyscallCommand> {
    unsafe {
        let status_b = SYSCALL_IN_MEM as *mut u8;
        if status_b.read() == 0 {
            None
        } else {
            let cmd: &mut SyscallCommand = &mut *((SYSCALL_IN_MEM + 1) as *mut SyscallCommand);
            let cmd = cmd.clone();
            status_b.write(0);
            Some(cmd)
        }
    }
}

pub fn run_command(command: SyscallCommand) {
    unsafe {
        (SYSCALL_IN_MEM as *mut u8).write(255);
        let cmd: &mut SyscallCommand = &mut *((SYSCALL_IN_MEM + 1) as *mut SyscallCommand);
        *cmd = command;
        asm!("int 80", options(nomem, nostack));
    }
}

pub enum SyscallMessage {
    Test,
}

pub struct SyscallSync {
    pub magic: u8,
    pub message_handler: fn(SyscallMessage),
}

fn message_handler(_msg: SyscallMessage) {
    run_command(SyscallCommand::Test1);
}

unsafe fn set_syscall_sync(data: SyscallSync) {
    let place: &mut SyscallSync = &mut *(SYSCALL_SYNC_MEM as *mut SyscallSync);
    *place = data;
}

pub unsafe fn call_handler(msg: SyscallMessage) {
    let place: &mut SyscallSync = &mut *(SYSCALL_SYNC_MEM as *mut SyscallSync);
    if place.magic == 255 {
        (place.message_handler)(msg)
    }
}

pub fn init() {
    unsafe {
        set_syscall_sync(SyscallSync {
            magic: 255,
            message_handler,
        });
    }
}
