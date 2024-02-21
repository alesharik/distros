use pci_types::{ConfigRegionAccess, PciAddress, PciHeader};

mod device;

pub use device::{PciDeviceBarMessage, PciDeviceStatusMessage, PciDeviceTypeMessage};

fn check_function(segment: u16, bus: u8, device: u8, function: u8) {
    let address = PciAddress::new(segment, bus, device, function);
    // if let Err(error) = device::register(address, distros_pci_access::access()) {
    //     error!(
    //         "[PCI][{:02}:{:02}.{:02}] Failed to create PCI device: {:?}",
    //         bus, device, function, error
    //     );
    // }
}

fn check_device(segment: u16, bus: u8, device: u8) {
    let pci_header = PciHeader::new(PciAddress::new(0, bus, device, 0));
    let (vendor_id, _) = pci_header.id(&distros_pci_access::access());
    if vendor_id == 0xFFFF {
        return;
    }
    check_function(segment, bus, device, 0);
    if pci_header.has_multiple_functions(&distros_pci_access::access()) {
        for fun in 1..8 {
            if PciHeader::new(PciAddress::new(0, bus, device, fun))
                .id(&distros_pci_access::access())
                .0
                != 0xFFFF
            {
                check_function(segment, bus, device, fun);
            }
        }
    }
}

fn check_bus(segment: u16, bus: u8) {
    for device in 0..32 {
        check_device(segment, bus, device);
    }
}

pub fn init() {
    for bus in 0..32 {
        check_bus(0, bus);
    }
}
