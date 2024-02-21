#![no_std]
#![feature(allocator_api)]

#[macro_use]
extern crate alloc;

use crate::pci::PciAccess;
use crate::pcie::PcieAccess;
use log::info;
use pci_types::{ConfigRegionAccess, PciAddress};

mod pci;
mod pcie;

static PCI_ACCESS: PciAccess = PciAccess::new();
static mut PCIE_ACCESS: Option<PcieAccess> = None;

pub struct AccessImpl;

impl ConfigRegionAccess for AccessImpl {
    fn function_exists(&self, address: PciAddress) -> bool {
        unsafe {
            if let Some(pcie) = PCIE_ACCESS.as_ref() {
                pcie.function_exists(address)
            } else {
                PCI_ACCESS.function_exists(address)
            }
        }
    }

    unsafe fn read(&self, address: PciAddress, offset: u16) -> u32 {
        unsafe {
            if let Some(pcie) = PCIE_ACCESS.as_ref() {
                pcie.read(address, offset)
            } else {
                PCI_ACCESS.read(address, offset)
            }
        }
    }

    unsafe fn write(&self, address: PciAddress, offset: u16, value: u32) {
        unsafe {
            if let Some(pcie) = PCIE_ACCESS.as_ref() {
                pcie.write(address, offset, value)
            } else {
                PCI_ACCESS.write(address, offset, value)
            }
        }
    }
}

pub fn init() {
    if let Some(regs) = distros_acpi::pci_config_regions() {
        unsafe {
            PCIE_ACCESS = Some(PcieAccess::new(regs));
        }
        info!("Will use PCIe");
    } else {
        info!("Will use legacy PCI");
    }
}

pub fn access() -> AccessImpl {
    AccessImpl
}
