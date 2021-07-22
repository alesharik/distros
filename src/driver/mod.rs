mod device;
pub mod keyboard;
pub mod mouse;
mod pci;
mod syslog;
mod tty;
mod smbios;

pub use tty::TtyMessage;
pub use syslog::SyslogMessage;

pub fn init() {
    syslog::init();

    kblog!("Driver", "Starting device drivers");
    device::init();
    kblog!("Driver", "Device drivers started");

    smbios::init();
    pci::print();
    tty::init().unwrap();
}
