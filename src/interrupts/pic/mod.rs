use acpi::platform::interrupt::Apic;
use x86_64::PhysAddr;

mod ioapic;
mod lapic;
mod nmi;
mod pic8259;

use core::sync::atomic::{AtomicBool, Ordering};
use x86_64::structures::paging::{Page, PageTableFlags, PhysFrame, Size4KiB};
use distros_memory::translate_kernel;
pub use ioapic::{convert_isr_irq, map_irc_irq};
pub use lapic::{eoi, invoke_lapic_timer_interrupt, start_lapic_timer};
pub use nmi::{nmi_status, StatusA, StatusB};

static INT_ENABLED: AtomicBool = AtomicBool::new(false);

pub fn init_pic(apic: &Apic) {
    pic8259::disable();
    if !crate::cpuid::has_apic() {
        panic!("Hardware does not have APIC")
    }
    let addr = PhysAddr::new(apic.local_apic_address);
    let virt = translate_kernel(addr);
    distros_memory::map(
        PhysFrame::<Size4KiB>::containing_address(addr),
        Page::containing_address(virt),
        PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::NO_CACHE | PageTableFlags::NO_EXECUTE
    ).unwrap();
    lapic::init_lapic(virt);
    ioapic::init_ioapic(&apic);
}

pub fn enable_interrupts() {
    INT_ENABLED.store(true, Ordering::SeqCst);
    x86_64::instructions::interrupts::enable();
    nmi::nmi_enable();
}

pub fn disable_interrupts() {
    x86_64::instructions::interrupts::disable();
    nmi::nmi_disable();
    INT_ENABLED.store(false, Ordering::SeqCst);
}

pub fn no_int<F, R>(f: F) -> R
where
    F: FnOnce() -> R,
{
    if !INT_ENABLED.load(Ordering::SeqCst) {
        return f();
    }
    INT_ENABLED.store(false, Ordering::SeqCst);
    x86_64::instructions::interrupts::disable();
    nmi::nmi_disable();
    let v = f();
    INT_ENABLED.store(true, Ordering::SeqCst);
    nmi::nmi_enable();
    x86_64::instructions::interrupts::enable();
    v
}
