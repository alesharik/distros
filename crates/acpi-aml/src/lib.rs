#![no_std]

extern crate alloc;

use crate::handler::HandlerImpl;
use acpi::{AcpiError, AmlTable};
use alloc::boxed::Box;
use alloc::vec::Vec;
use aml::{AmlContext, DebugVerbosity};
use core::ptr::slice_from_raw_parts;
use core::slice;
use distros_memory::translate_kernel;
use log::{error, info, warn};
use x86_64::PhysAddr;

mod handler;

static mut CTX: Option<AmlContext> = None;

pub fn init() {
    let mut context = AmlContext::new(Box::new(HandlerImpl), DebugVerbosity::All);
    unsafe {
        match distros_acpi::parse_dsdt() {
            Ok(dsdt) => {
                let addr = PhysAddr::new(dsdt.address as u64);
                let addr = translate_kernel(addr);
                let data: &[u8] = slice::from_raw_parts(addr.as_ptr(), dsdt.length as usize);
                if let Err(e) = context.parse_table(data) {
                    error!("Failed to parse DSDT: {:?}", e);
                    return;
                }
            }
            Err(e) => {
                error!("Failed to load DSDT: {:?}", e);
                return;
            }
        }
    }

    unsafe {
        for table in distros_acpi::parse_ssdts() {
            let addr = PhysAddr::new(table.address as u64);
            let addr = translate_kernel(addr);
            let data = slice::from_raw_parts(addr.as_ptr(), table.length as usize);
            if let Err(e) = context.parse_table(&data as &[u8]) {
                error!("Failed to parse SSDT: {:?}", e);
                return;
            }
        }
    }

    if let Err(e) = context.initialize_objects() {
        error!("Failed to initialize AML objects: {:?}", e);
        return;
    }
    unsafe {
        CTX = Some(context);
    }
    info!("AML initialized");
}
