#![no_std]
#![feature(abi_x86_interrupt)]

use core::sync::atomic::{AtomicU64, Ordering};
use core::time::Duration;
use distros_interrupt::int_handler;
use log::warn;
use x86_64::structures::idt::InterruptStackFrame;

mod cmos;
mod rtc;

static TIME: AtomicU64 = AtomicU64::new(0);
static mut DELAY: Duration = Duration::from_secs(0);

pub struct ExternalTimerInfo {
    pub delay: Duration,
}

pub fn init(external_timer: Option<ExternalTimerInfo>) {
    let time = loop {
        if let Some(time) = cmos::read_time() {
            break time;
        }
    };
    TIME.store(time.timestamp_millis() as u64, Ordering::SeqCst);
    match external_timer {
        None => {
            distros_timer_tsc::tsc_calibration_frequency(Duration::from_nanos(976562));
            unsafe {
                DELAY = Duration::from_millis(1);
            }
            rtc::init_rtc();
        }
        Some(tim) => {
            distros_timer_tsc::tsc_calibration_frequency(tim.delay);
            unsafe {
                DELAY = tim.delay;
            }
        }
    }
}

int_handler!(pub noint rtc_handler |_frame: InterruptStackFrame| {
    distros_timer_tsc::tsc_calibration_sample();
    let ms = TIME.fetch_add(unsafe { DELAY.as_millis() } as u64, Ordering::AcqRel);
    if ms % 100 == 0 { // Every 100 ms
        let ms_part = ms % 1000;
        let time = cmos::read_time_unsafe();
        if let Some(time) = time {
            TIME.store(time.timestamp_millis() as u64 + ms_part, Ordering::Relaxed);
        }
    };

    rtc::eoi();
    distros_interrupt_pic::lapic_eoi();
});

#[inline]
pub fn now() -> u64 {
    TIME.load(Ordering::Acquire)
}
