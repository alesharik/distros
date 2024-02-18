use core::time::Duration;

mod hpet;
mod pit;

use distros_interrupt::OverrideMode;
use x86_64::structures::idt::InterruptStackFrame;

pub fn init_timer() {
    if let Some(hpet) = distros_acpi::hpet() {
        distros_timer_rtc::init(Some(1000));
        let irq = hpet::init_hpet_rtc(hpet);
        distros_interrupt::set_handler(
            irq.map_to_int(0),
            distros_timer_rtc::rtc_handler,
            OverrideMode::Panic,
        );
        info!("Handler mapped to irq {} via HPET", irq.0);
        hpet::start_hpet(hpet);

        let pit_irq = pit::PIT_IRQ;
        if !pit_irq.has_handler() {
            distros_interrupt::set_handler(pit_irq.map_to_int(0), pit_stub, OverrideMode::Panic);
        }
    } else {
        // let pit_irq = pit::init_pit();
        // crate::pic::map_irc_irq(pit_irq, 0);

        distros_timer_rtc::init(None);
    }
}

int_handler!(
    pit_stub | _: InterruptStackFrame | {
        distros_interrupt_pic::lapic_eoi();
    }
);
