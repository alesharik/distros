mod device;
pub mod keyboard;
pub mod mouse;
mod pci;
mod smbios;
mod syslog;
mod tty;

pub use pci::{PciDeviceBarMessage, PciDeviceTypeMessage};
pub use syslog::SyslogMessage;
pub use tty::TtyMessage;
use crate::acpi::AcpiInfo;

pub fn init(acpi: &AcpiInfo) {
    syslog::init();

    kblog!("Driver", "Starting device drivers");
    device::init();
    kblog!("Driver", "Device drivers started");

    smbios::init();
    pci::init(acpi);
    tty::init().unwrap();
}
