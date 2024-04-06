#![no_std]

extern crate alloc;

mod buffer;

use x86_64::VirtAddr;

pub const KERNEL_STACK_BASE_GUARD: VirtAddr = VirtAddr::new_truncate(0xf_0000_0000);
pub const KERNEL_STACK_BASE: VirtAddr = VirtAddr::new_truncate(0xf_0000_0000 + 0x1000u64);
pub const KERNEL_STACK_SIZE: u64 = 80 * 1024; // 80 KiB

pub use buffer::{find_buffer, make_copy, new_buffer, StackBuffer, StackBufferHandle};
