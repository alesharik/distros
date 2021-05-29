use x86_64::structures::idt::InterruptStackFrame;
use core::sync::atomic::{AtomicU64, Ordering};
use x86_64::instructions::port::{PortWriteOnly, Port};
use spin::Mutex;

const RATE: u8 = 6 & 0x0F;

static TIME: AtomicU64 = AtomicU64::new(0);

lazy_static!(
    static ref ADDRESS: Mutex<PortWriteOnly<u8>> = Mutex::new(PortWriteOnly::<u8>::new(0x70));
    static ref DATA: Mutex<Port<u8>> = Mutex::new(Port::<u8>::new(0x71));
);

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

pub extern "x86-interrupt" fn rtc_handler(_stack_frame: &mut InterruptStackFrame)  {
    crate::interrupts::no_int(|| {
        let ms = TIME.fetch_add(1, Ordering::AcqRel);
        if ms % 500 == 0 { // Every 500 ms
            let ms_part = ms % 1000;
            let time = crate::cmos::read_time_unsafe();
            TIME.store((time.timestamp_millis() as u64 + ms_part) as u64, Ordering::Relaxed)
        };

        unsafe {
            let mut data = DATA.lock();
            let mut address = ADDRESS.lock();
            address.write(0x0C);
            data.read(); // Ignore. Resets IRQ
            crate::interrupts::eoi();
        }
    })
}

#[inline]
pub fn now() -> u64 {
    TIME.load(Ordering::Acquire)
}