#![no_std]

use crate::hpet::Hpet;
use crate::timer::Timer;
use acpi::platform::interrupt::TriggerMode;
use core::time::Duration;
use distros_interrupt_pic::{Irq, IrqDestination, IrqId, IrqMode};
use distros_memory::translate_kernel;
use distros_timer_rtc::{rtc_handler, ExternalTimerInfo};
use log::{debug, info, warn};
use spin::Mutex;
use x86_64::structures::paging::{Page, PageTableFlags, PhysFrame, Size4KiB};
use x86_64::{PhysAddr, VirtAddr};

mod capabilities;
mod hpet;
mod timer;

static mut HPET: Option<Mutex<Hpet>> = None;

pub fn init(info: &acpi::hpet::HpetInfo) {
    let phys = PhysAddr::new(info.base_address as u64);
    let addr: VirtAddr = translate_kernel(phys);
    distros_memory::map(
        PhysFrame::<Size4KiB>::containing_address(phys),
        Page::containing_address(addr),
        PageTableFlags::PRESENT
            | PageTableFlags::WRITABLE
            | PageTableFlags::NO_CACHE
            | PageTableFlags::NO_EXECUTE,
    )
    .unwrap();
    let mut hpet = Hpet::new(addr);
    hpet.disable();
    hpet.disable_legacy_replacement();
    hpet.set_main_counter(0);
    info!("HPET loaded");

    let timer = hpet.iter().find(|t| t.supports_periodic());
    match timer {
        None => {
            distros_timer_rtc::init(None);
            warn!("HPET does not have any periodic timers! RTC set up with own timer");
        }
        Some(timer) => {
            let duration = Duration::from_millis(50); // lower value will not work in QEMU
            let comp = (duration.as_nanos() as u64 * 10_u64.pow(6)) / hpet.period() as u64;
            let handler = distros_interrupt::alloc_handler(rtc_handler)
                .expect("Failed to alloc new interrupt");
            let line = timer
                .first_available_apic_line()
                .expect("Failed to find APIC line");
            let irq: Irq = Irq::new(IrqId::new(line as u32)).trigger_mode(TriggerMode::Edge);
            distros_interrupt_pic::ioapic_set_entry(
                irq,
                IrqDestination::Local,
                handler,
                IrqMode::Fixed,
            )
            .expect("Failed to register IOAPIC entry");
            distros_interrupt_pic::ioapic_enable(irq.get_global_system_interrupt())
                .expect("Failed to register IOAPIC entry");
            let timer = timer
                .set_periodic(true)
                .set_interrupts_enabled(true)
                .set_apic_line(line)
                .allow_set_accumulator();
            hpet.set_timer(timer);
            hpet.set_timer_comparator(timer, comp);
            hpet.set_timer_comparator(timer, 0);
            distros_timer_rtc::init(Some(ExternalTimerInfo { delay: duration }));
            info!("RTC set up with HPET {:?}", &timer);
        }
    }

    unsafe {
        HPET = Some(Mutex::new(hpet));
    }
}

pub fn enable() {
    unsafe {
        let mut hpet = HPET.as_ref().expect("HPET not initialized").lock();
        hpet.enable();
    }
    debug!("HPET enabled");
}
