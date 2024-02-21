mod device;
pub mod keyboard;
pub mod mouse;
pub mod pci;
mod smbios;
mod syslog;
mod tty;
// pub use pci::{PciDeviceBarMessage, PciDeviceTypeMessage};
pub use syslog::SyslogMessage;
pub use tty::TtyMessage;

pub fn init() {
    syslog::init();

    info!("Starting device drivers");
    device::init();
    info!("Device drivers started");

    smbios::init();
    // pci::init();
    tty::init().unwrap();
}
