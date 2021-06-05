use pci_types::{PciAddress, PciHeader, ConfigRegionAccess};
use x86_64::instructions::port::{PortWriteOnly, PortReadOnly, Port};
use core::cell::RefCell;
use pci_types::device_type::DeviceType;
use bit_field::BitField;

struct AccessImpl {
    address_port: RefCell<PortWriteOnly<u32>>,
    data_port: RefCell<Port<u32>>,
}

impl AccessImpl {
    fn new() -> AccessImpl {
        AccessImpl {
            address_port: RefCell::new(PortWriteOnly::new(0xCF8)),
            data_port: RefCell::new(Port::new(0xCFC))
        }
    }
}

impl ConfigRegionAccess for AccessImpl {
    fn function_exists(&self, address: PciAddress) -> bool {
        unsafe {
            self.read(address, 0) & 0xFFFF != 0xFFFF
        }
    }

    unsafe fn read(&self, address: PciAddress, offset: u16) -> u32 {
        let mut result: u32 = 0;
        result.set_bits(0..8, offset as u32);
        result.set_bits(8..11, address.function() as u32);
        result.set_bits(11..16, address.device() as u32);
        result.set_bits(16..23, address.bus() as u32);
        result.set_bit(31, true);
        self.address_port.borrow_mut().write(result);
        self.data_port.borrow_mut().read()
    }

    unsafe fn write(&self, address: PciAddress, offset: u16, value: u32) {
        let mut result = 0u32;
        result.set_bits(0..8, offset as u32);
        result.set_bits(8..11, address.function() as u32);
        result.set_bits(11..16, address.device() as u32);
        result.set_bits(16..23, address.bus() as u32);
        result.set_bit(31, true);
        self.address_port.borrow_mut().write(result);
        self.data_port.borrow_mut().write(value);
    }
}

fn check_function(access: &AccessImpl, bus: u8, device: u8, function: u8) {
    let header = PciHeader::new(PciAddress::new(0, bus, device, function));
    let id = header.id(access);
    let (_, base, sub, _) = header.revision_and_class(access);
    println!("PCI {:?} => {:?}", id, DeviceType::from((base, sub)));
}

fn check_device(access: &AccessImpl, bus: u8, device: u8) {
    let pci_header = PciHeader::new(PciAddress::new(0, bus, device, 0));
    let (vendor_id, _) = pci_header.id(access);
    if vendor_id == 0xFFFF {
        return
    }
    check_function(access, bus, device, 0);
    if pci_header.has_multiple_functions(access) {
        for fun in 1..8 {
            if PciHeader::new(PciAddress::new(0, bus, device, fun)).id(access).0 != 0xFFFF {
                check_function(access, bus, device, fun);
            }
        }
    }
}

fn check_bus(access: &AccessImpl, bus: u8) {
    for device in 0..32 {
        check_device(access, bus, device);
    }
}

pub fn print() {
    let access = AccessImpl::new();
    for bus in 0..32 {
        check_bus(&access, bus);
    }
}