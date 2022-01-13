#![no_std]
#![feature(asm)]
#![feature(ptr_metadata)]

extern crate alloc;

pub mod flow;
pub mod syscall;
pub mod task;