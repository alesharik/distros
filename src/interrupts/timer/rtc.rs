use core::sync::atomic::{AtomicU64, Ordering};
use spin::Mutex;
use x86_64::instructions::port::{Port, PortWriteOnly};
use x86_64::structures::idt::InterruptStackFrame;

const RATE: u8 = 6 & 0x0F;

static TIME: AtomicU64 = AtomicU64::new(0);

lazy_static! {
    static ref ADDRESS: Mutex<PortWriteOnly<u8>> = Mutex::new(PortWriteOnly::<u8>::new(0x70));
    static ref DATA: Mutex<Port<u8>> = Mutex::new(Port::<u8>::new(0x71));
}

pub fn start_rtc() {
    let start_time = crate::cmos::read_time();
    TIME.store(start_time.timestamp_millis() as u64, Ordering::Release);
    unsafe {
        let mut data = DATA.lock();
        let mut address = ADDRESS.lock();
        address.write(0x8B);
        let prev = data.read();
        address.write(0x8B);
        data.write((prev | 0x40) & 0xF0 | RATE);
        // address.write(0x0C);
        // data.read(); // Ignore. Resets IRQ
    }
    kblog!("RTC", "RTC started");
}

int_handler!(pub noint rtc_handler |_frame: InterruptStackFrame| {
    let ms = TIME.fetch_add(1, Ordering::AcqRel);
    if ms % 500 == 0 { // Every 500 ms
        let ms_part = ms % 1000;
        let time = crate::cmos::read_time_unsafe();
        TIME.store((time.timestamp_millis() as u64 + ms_part) as u64, Ordering::Relaxed)
    };
    crate::process::sleep::tick_1ms();
    unsafe {
        let mut data = DATA.lock();
        let mut address = ADDRESS.lock();
        address.write(0x0C);
        data.read(); // Ignore. Resets IRQ
    }
});

#[inline]
pub fn now() -> u64 {
    TIME.load(Ordering::Acquire)
}
