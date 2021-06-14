mod device;
pub mod keyboard;
pub mod mouse;
mod pci;
mod syslog;
mod tty;

pub use tty::TtyMessage;

pub fn init() {
    syslog::init();

    kblog!("Driver", "Starting device drivers");
    device::init();
    kblog!("Driver", "Device drivers started");

    pci::print();
    tty::init().unwrap();
}
