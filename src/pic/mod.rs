use acpi::platform::Apic;
use x86_64::PhysAddr;

mod pic8259;
mod lapic;
mod ioapic;
mod nmi;

pub use ioapic::map_irc_irq;
pub use lapic::eoi;
pub use nmi::{nmi_status, StatusA, StatusB};

pub fn init_pic(apic: &Apic) {
    pic8259::disable();
    if !crate::cpuid::has_apic() {
        panic!("Hardware does not have APIC")
    }
    lapic::init_lapic(crate::memory::map_physical_address(PhysAddr::new(apic.local_apic_address)));
    ioapic::init_ioapic(&apic);
}

pub fn enable_interrupts() {
    x86_64::instructions::interrupts::enable();
    nmi::nmi_enable();
}

pub fn no_int<F, R>(f: F) -> R
    where
        F: FnOnce() -> R {
    nmi::nmi_disable();
    x86_64::instructions::interrupts::disable();
    let v = f();
    x86_64::instructions::interrupts::enable();
    nmi::nmi_enable();
    v
}
