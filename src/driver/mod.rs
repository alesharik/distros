mod device;
pub mod keyboard;
pub mod mouse;
mod pci;
mod smbios;
mod syslog;
mod tty;
use crate::acpi::AcpiInfo;
pub use pci::{PciDeviceBarMessage, PciDeviceTypeMessage};
pub use syslog::SyslogMessage;
pub use tty::TtyMessage;

pub fn init(acpi: &AcpiInfo) {
    syslog::init();

    info!("Starting device drivers");
    device::init();
    info!("Device drivers started");

    smbios::init();
    pci::init(acpi);
    tty::init().unwrap();
}
