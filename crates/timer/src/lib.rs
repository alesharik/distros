#![no_std]

use core::time::Duration;
pub use distros_timer_rtc::now;
pub use distros_timer_tsc::uptime;
use log::info;

static mut USE_HPET: bool = false;

pub fn init() {
    if let Some(hpet) = distros_acpi::hpet() {
        distros_timer_hpet::init(hpet);
        unsafe { USE_HPET = true };
        info!("HPET initialized, will use it for all timing stuff")
    } else {
        distros_timer_rtc::init(None);
        info!("HPET not found, use RTC for interrupt, TSC for timing")
    }
}

pub fn after_interrupt_enabled() {
    if distros_acpi::hpet().is_some() {
        distros_timer_hpet::enable();
    }
}

pub fn sleep(duration: Duration) {
    unsafe {
        if USE_HPET {
            distros_timer_hpet::sleep(duration);
        } else {
            distros_timer_tsc::sleep(duration);
        }
    }
}
