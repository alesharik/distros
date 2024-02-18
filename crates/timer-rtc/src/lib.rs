#![no_std]
#![feature(abi_x86_interrupt)]

use core::sync::atomic::{AtomicU64, Ordering};
use distros_interrupt::int_handler;
use spin::RwLock;
use x86_64::structures::idt::InterruptStackFrame;

mod cmos;
mod rtc;

static TIME: AtomicU64 = AtomicU64::new(0);
static MILLISECOND_HANDLER: RwLock<Option<fn()>> = RwLock::new(None);

pub fn init(load_rtc: bool) {
    let time = loop {
        if let Some(time) = cmos::read_time() {
            break time;
        }
    };
    TIME.store(time.timestamp_millis() as u64, Ordering::SeqCst);
    if load_rtc {
        rtc::init_rtc();
    }
}

/// Should invoke every 1ms
int_handler!(pub noint rtc_handler |_frame: InterruptStackFrame| {
    let ms = TIME.fetch_add(1, Ordering::AcqRel);
    if ms % 100 == 0 { // Every 100 ms
        let ms_part = ms % 1000;
        let time = cmos::read_time_unsafe();
        if let Some(time) = time {
            TIME.store(time.timestamp_millis() as u64 + ms_part, Ordering::Relaxed)
        }
    };

    let handler = MILLISECOND_HANDLER.read();
    if let Some(h) = handler.as_ref() {
        h()
    }

    rtc::eoi();
    distros_interrupt_pic::lapic_eoi();
});

#[inline]
pub fn now() -> u64 {
    TIME.load(Ordering::Acquire)
}

pub fn set_millisecond_handler(h: fn()) {
    let mut handler = MILLISECOND_HANDLER.write();
    *handler = Some(h);
}
