use crate::flow::FlowManagerError;
use pci_types::{Bar, ConfigRegionAccess, EndpointHeader, MAX_BARS, PciAddress, PciHeader, StatusRegister};
use pci_types::device_type::DeviceType;
use libkernel::flow::{U16Message, U8Message};

type Result<T> = core::result::Result<T, FlowManagerError>;

primitive_message!(PciDeviceTypeMessage DeviceType);
primitive_message!(PciDeviceBarMessage Bar);
primitive_message!(PciDeviceStatusMessage StatusRegister);

fn register_status<T: ConfigRegionAccess + Sync + Clone + 'static>(address: PciAddress, access: &T) -> Result<()> {
    let access = access.clone();
    let header = PciHeader::new(address);
    register!(dynacontent format!("/dev/pci/{}/{}/{}/status", address.bus(), address.device(), address.function()) => PciDeviceStatusMessage move || {
        PciDeviceStatusMessage::new(header.status(&access))
    });
    Ok(())
}

pub fn register<T: ConfigRegionAccess + Sync + Clone + 'static>(address: PciAddress, access: &T) -> Result<()> {
    let header = PciHeader::new(address);
    let (vendor, device_id) = header.id(access);
    let (revision, base, sub, interface) = header.revision_and_class(access);
    register!(content format!("/dev/pci/{}/{}/{}/header_type", address.bus(), address.device(), address.function()) => U8Message (header.header_type(access)));
    register!(content format!("/dev/pci/{}/{}/{}/vendor", address.bus(), address.device(), address.function()) => U16Message (vendor));
    register!(content format!("/dev/pci/{}/{}/{}/device", address.bus(), address.device(), address.function()) => U16Message (device_id));
    register!(content format!("/dev/pci/{}/{}/{}/revision", address.bus(), address.device(), address.function()) => U8Message (revision));
    register!(content format!("/dev/pci/{}/{}/{}/base_class", address.bus(), address.device(), address.function()) => U8Message (base));
    register!(content format!("/dev/pci/{}/{}/{}/sub_class", address.bus(), address.device(), address.function()) => U8Message (sub));
    register!(content format!("/dev/pci/{}/{}/{}/interface", address.bus(), address.device(), address.function()) => U8Message (interface));
    register!(content format!("/dev/pci/{}/{}/{}/type", address.bus(), address.device(), address.function()) => PciDeviceTypeMessage (DeviceType::from((base, sub))));

    register_status(address, access)?;

    if let Some(endpoint) = EndpointHeader::from_header(header, access) {
        for bar_id in 0..(MAX_BARS as u8) {
            if let Some(bar) = endpoint.bar(bar_id, access) {
                register!(content format!("/dev/pci/{}/{}/{}/bar/{}", address.bus(), address.device(), address.function(), bar_id) => PciDeviceBarMessage(bar));
            }
        }

        for capability in endpoint.capabilities(access) {
            // register!(content format!("/dev/pci/{}/{}/{}/capability/{}", bus, device, function, capability) => PciDeviceBarMessage(bar));
            println!("{:?}", capability);
        }
    }
    Ok(())
}