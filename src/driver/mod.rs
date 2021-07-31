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

pub fn init() {
    syslog::init();

    kblog!("Driver", "Starting device drivers");
    device::init();
    kblog!("Driver", "Device drivers started");

    smbios::init();
    pci::init();
    tty::init().unwrap();
}
