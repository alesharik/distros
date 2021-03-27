use x86_64::structures::idt::InterruptStackFrame;
use core::sync::atomic::{AtomicU64, Ordering};
use core::time::Duration;

mod pit;
mod rtc;

pub use rtc::now;

static mut TIME: AtomicU64 = AtomicU64::new(0);

extern "x86-interrupt" fn ktimer_handler(_stack_frame: &mut InterruptStackFrame) {
    unsafe { TIME.fetch_add(1, Ordering::AcqRel); }
    crate::pic::eoi();
}

pub fn sleep(duration: Duration) {
    unsafe {
        let delta = duration.as_millis();
        let now = TIME.load(Ordering::Acquire);
        while TIME.load(Ordering::Acquire) - now < delta as u64 {
            x86_64::instructions::hlt();
        }
    }
}

pub fn init_timer() {
    let pit_irq = pit::init_pit();
    let pit_mapped_irq = crate::pic::map_irc_irq(pit_irq, 0);
    crate::interrupts::set_handler(pit_mapped_irq, ktimer_handler);
    kblog!("Timer", "KTimer started");

    let rtc_mapped_irq = crate::pic::map_irc_irq(rtc::IRQ, 0);
    crate::interrupts::set_handler(rtc_mapped_irq, rtc::rtc_handler);
    rtc::start_rtc();
}