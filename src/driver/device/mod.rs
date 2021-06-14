/// this module contains real device drivers
mod ps2;

pub fn init() {
    if let Err(e) = ps2::init() {
        kblog!("Drivers/Device", "Failed to init PS/2 controller: {:?}", e);
    }
}
