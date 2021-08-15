#![no_std]
#![no_main]

use core::panic::PanicInfo;
use libkernel::syscall::{run_command, SyscallCommand};

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    loop {}
}

#[no_mangle]
fn _start() -> ! {
    libkernel::syscall::init();
    run_command(SyscallCommand::Test);
    loop {
    }
}
