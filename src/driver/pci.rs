use crate::flow::FlowManagerError;
use bit_field::BitField;
use core::cell::RefCell;
use core::option::Option::Some;
use libkernel::flow::{U16Message, U8Message};
use pci_types::device_type::DeviceType;
use pci_types::MAX_BARS;
use pci_types::{Bar, ConfigRegionAccess, EndpointHeader, PciAddress, PciHeader};
use x86_64::instructions::port::{Port, PortWriteOnly};

primitive_message!(PciDeviceTypeMessage DeviceType);
primitive_message!(PciDeviceBarMessage Bar);

type Result<T> = core::result::Result<T, FlowManagerError>;

struct AccessImpl {
    address_port: RefCell<PortWriteOnly<u32>>,
    data_port: RefCell<Port<u32>>,
}

impl AccessImpl {
    fn new() -> AccessImpl {
        AccessImpl {
            address_port: RefCell::new(PortWriteOnly::new(0xCF8)),
            data_port: RefCell::new(Port::new(0xCFC)),
        }
    }
}

impl ConfigRegionAccess for AccessImpl {
    fn function_exists(&self, address: PciAddress) -> bool {
        unsafe { self.read(address, 0) & 0xFFFF != 0xFFFF }
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

fn create_device(
    access: &AccessImpl,
    header: PciHeader,
    bus: u8,
    device: u8,
    function: u8,
) -> Result<()> {
    let (vendor, device_id) = header.id(access);
    let (revision, base, sub, interface) = header.revision_and_class(access);
    register!(content format!("/dev/pci/{}/{}/{}/header_type", bus, device, function) => U8Message (header.header_type(access)));
    register!(content format!("/dev/pci/{}/{}/{}/vendor", bus, device, function) => U16Message (vendor));
    register!(content format!("/dev/pci/{}/{}/{}/device", bus, device, function) => U16Message (device_id));
    register!(content format!("/dev/pci/{}/{}/{}/revision", bus, device, function) => U8Message (revision));
    register!(content format!("/dev/pci/{}/{}/{}/base_class", bus, device, function) => U8Message (base));
    register!(content format!("/dev/pci/{}/{}/{}/sub_class", bus, device, function) => U8Message (sub));
    register!(content format!("/dev/pci/{}/{}/{}/interface", bus, device, function) => U8Message (interface));
    register!(content format!("/dev/pci/{}/{}/{}/type", bus, device, function) => PciDeviceTypeMessage (DeviceType::from((base, sub))));
    if let Some(endpoint) = EndpointHeader::from_header(header, access) {
        for bar_id in 0..(MAX_BARS as u8) {
            if let Some(bar) = endpoint.bar(bar_id, access) {
                register!(content format!("/dev/pci/{}/{}/{}/bar/{}", bus, device, function, bar_id) => PciDeviceBarMessage(bar));
            }
        }
    }
    Ok(())
}

fn check_function(access: &AccessImpl, bus: u8, device: u8, function: u8) {
    let header = PciHeader::new(PciAddress::new(0, bus, device, function));
    if let Err(error) = create_device(access, header, bus, device, function) {
        error!(
            "[PCI][{:02}:{:02}.{:02}] Failed to create PCI device: {:?}",
            bus, device, function, error
        );
    }
}

fn check_device(access: &AccessImpl, bus: u8, device: u8) {
    let pci_header = PciHeader::new(PciAddress::new(0, bus, device, 0));
    let (vendor_id, _) = pci_header.id(access);
    if vendor_id == 0xFFFF {
        return;
    }
    check_function(access, bus, device, 0);
    if pci_header.has_multiple_functions(access) {
        for fun in 1..8 {
            if PciHeader::new(PciAddress::new(0, bus, device, fun))
                .id(access)
                .0
                != 0xFFFF
            {
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

pub fn init() {
    let access = AccessImpl::new();
    for bus in 0..32 {
        check_bus(&access, bus);
    }
}
