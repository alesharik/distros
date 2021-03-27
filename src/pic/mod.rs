use acpi::platform::Apic;
use x86_64::{PhysAddr, VirtAddr};

mod pic8259;
mod lapic;
mod ioapic;

pub use ioapic::map_irc_irq;
pub use lapic::eoi;

pub fn init_pic(apic: &Apic) {
    pic8259::disable();
    if !crate::cpuid::has_apic() {
        panic!("Hardware does not have APIC")
    }
    lapic::init_lapic(crate::memory::map_physical_address(PhysAddr::new(apic.local_apic_address)));
    ioapic::init_ioapic(&apic);
}

pub fn enable_interrupts() {
    unsafe {
        x86_64::instructions::interrupts::enable();
    }
}

pub fn no_int<F, R>(f: F) -> R
    where
        F: FnOnce() -> R {
    unsafe {
        x86_64::instructions::interrupts::disable();
    }
    let v = f();
    unsafe {
        x86_64::instructions::interrupts::enable();
    }
    v
}
