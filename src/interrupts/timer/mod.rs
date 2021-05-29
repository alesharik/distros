use core::time::Duration;

mod pit;
mod rtc;
mod hpet;

pub use rtc::now;
use crate::acpi::AcpiInfo;
use crate::interrupts::RTC_IRQ;

pub fn sleep(duration: Duration) {
    let delta = duration.as_millis();
    let now = rtc::now();
    while rtc::now() - now < delta as u64 {
        x86_64::instructions::hlt();
    }
}

pub fn init_timer(acpi: &AcpiInfo) {
    if let Some(hpet) = acpi.hpet.as_ref() {
        let irq = hpet::init_hpet_rtc(hpet);
        crate::interrupts::set_handler(irq.map_to_int(0), rtc::rtc_handler);
        kblog!("[RTC]", "Handler mapped to irq {} via HPET", irq.0);
        hpet::start_hpet(hpet);
    } else {
        // let pit_irq = pit::init_pit();
        // crate::pic::map_irc_irq(pit_irq, 0);

        let rtc_mapped_irq = RTC_IRQ.map_to_int(0);
        crate::interrupts::set_handler(rtc_mapped_irq, rtc::rtc_handler);
        rtc::start_rtc();
    }
}