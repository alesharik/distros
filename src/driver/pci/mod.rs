use pci_types::{ConfigRegionAccess, PciAddress, PciHeader};

mod access;
mod device;

use crate::driver::pci::access::{PciAccess, PcieAccess};
pub use device::{PciDeviceBarMessage, PciDeviceStatusMessage, PciDeviceTypeMessage};

fn check_function<T: ConfigRegionAccess + Sync + Clone + 'static>(
    access: &T,
    segment: u16,
    bus: u8,
    device: u8,
    function: u8,
) {
    let address = PciAddress::new(segment, bus, device, function);
    if let Err(error) = device::register(address, access) {
        error!(
            "[PCI][{:02}:{:02}.{:02}] Failed to create PCI device: {:?}",
            bus, device, function, error
        );
    }
}

fn check_device<T: ConfigRegionAccess + Sync + Clone + 'static>(
    access: &T,
    segment: u16,
    bus: u8,
    device: u8,
) {
    let pci_header = PciHeader::new(PciAddress::new(0, bus, device, 0));
    let (vendor_id, _) = pci_header.id(access);
    if vendor_id == 0xFFFF {
        return;
    }
    check_function(access, segment, bus, device, 0);
    if pci_header.has_multiple_functions(access) {
        for fun in 1..8 {
            if PciHeader::new(PciAddress::new(0, bus, device, fun))
                .id(access)
                .0
                != 0xFFFF
            {
                check_function(access, segment, bus, device, fun);
            }
        }
    }
}

fn check_bus<T: ConfigRegionAccess + Sync + Clone + 'static>(access: &T, segment: u16, bus: u8) {
    for device in 0..32 {
        check_device(access, segment, bus, device);
    }
}

pub fn init() {
    if let Some(regions) = distros_acpi::pci_config_regions() {
        debug!("[PCI] Using PCIe interface");
        let access = PcieAccess::new(regions);
        for bus in 0..32 {
            check_bus(&access, 0, bus);
        }
    } else {
        debug!("[PCI] No MCFG table found! Using old PCI interface");
        let access = PciAccess::new();
        for bus in 0..32 {
            check_bus(&access, 0, bus);
        }
    }
}
