#![no_std]

#[macro_use]
extern crate alloc;

use alloc::vec::Vec;
use distros_pci_access::is_pcie;
use pci_types::{PciAddress, PciHeader};

struct Builder {
    vec: Vec<PciHeader>,
}

impl Builder {
    pub fn new() -> Self {
        Builder { vec: Vec::new() }
    }

    fn check_device(&mut self, segment: u16, bus: u8, device: u8) {
        let pci_header = PciHeader::new(PciAddress::new(segment, bus, device, 0));
        let (vendor_id, _) = pci_header.id(&distros_pci_access::access());
        if vendor_id == 0xFFFF {
            return;
        }
        self.vec
            .push(PciHeader::new(PciAddress::new(segment, bus, device, 0)));
        if pci_header.has_multiple_functions(&distros_pci_access::access()) {
            for fun in 1..8 {
                let header = PciHeader::new(PciAddress::new(segment, bus, device, fun));
                if header.id(&distros_pci_access::access()).0 != 0xFFFF {
                    self.vec.push(header);
                }
            }
        }
    }

    fn check_bus(&mut self, segment: u16, bus: u8) {
        for device in 0..32 {
            self.check_device(segment, bus, device);
        }
    }

    pub fn check_segment(mut self, segment: u16) -> Self {
        for bus in 0..32 {
            self.check_bus(segment, bus);
        }
        self
    }

    pub fn len(&self) -> usize {
        self.vec.len()
    }

    pub fn build(self) -> Vec<PciHeader> {
        self.vec
    }
}

pub fn vec() -> Vec<PciHeader> {
    if is_pcie() {
        let mut builder = Builder::new().check_segment(0);
        for i in 1..256 {
            let len = builder.len();
            builder = builder.check_segment(i);
            if len == builder.len() {
                break;
            }
        }
        builder.build()
    } else {
        Builder::new().check_segment(0).build()
    }
}
