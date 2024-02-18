use crate::rtc_handler;
use core::sync::atomic::{AtomicU64, Ordering};
use distros_interrupt::{int_handler, without_nmi};
use distros_interrupt_pic::{Irq, IrqDestination, IrqMode, IsaIrq};
use lazy_static::lazy_static;
use log::info;
use spin::Mutex;
use x86_64::instructions::interrupts::without_interrupts;
use x86_64::instructions::port::{Port, PortWriteOnly};
use x86_64::structures::idt::InterruptStackFrame;

const RATE: u8 = 6 & 0x0F; // 1024 hz
static mut ENABLED: bool = false;

lazy_static! {
    static ref ADDRESS: Mutex<PortWriteOnly<u8>> = Mutex::new(PortWriteOnly::<u8>::new(0x70));
    static ref DATA: Mutex<Port<u8>> = Mutex::new(Port::<u8>::new(0x71));
}

pub fn init_rtc() {
    let handler =
        distros_interrupt::alloc_handler(rtc_handler).expect("Failed to alloc new interrupt");
    let irq: Irq = IsaIrq::Rtc.into();
    distros_interrupt_pic::ioapic_set_entry(irq, IrqDestination::Local, handler, IrqMode::Fixed)
        .expect("Failed to register IOAPIC entry");
    unsafe {
        without_interrupts(|| {
            without_nmi(|| {
                let mut data = DATA.lock();
                let mut address = ADDRESS.lock();
                address.write(0x8B);
                let prev = data.read();
                address.write(0x8B);
                data.write((prev | 0x40) & 0xF0 | RATE);
            });
            distros_interrupt_pic::ioapic_enable(irq.get_global_system_interrupt())
                .expect("Failed to enable interrupt");
        });
        ENABLED = true;
    }
    info!("RTC started");
}

pub fn eoi() {
    unsafe {
        if !ENABLED {
            return;
        }
        without_interrupts(|| {
            without_nmi(|| {
                let mut data = DATA.lock();
                let mut address = ADDRESS.lock();
                address.write(0x0C);
                data.read(); // Ignore. Resets IRQ
            });
        });
    }
}
