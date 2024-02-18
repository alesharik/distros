#![no_std]

#[macro_use]
extern crate alloc;

use acpi::platform::interrupt::Apic;
use distros_memory::translate_kernel;
use x86_64::structures::paging::{Page, PageTableFlags, PhysFrame, Size4KiB};
use x86_64::PhysAddr;

mod lapic;
mod pic8259;

pub use lapic::{
    eoi as lapic_eoi, timer_disable as lapic_timer_disable, timer_enable as lapic_timer_enable,
    INT_LAPIC_TIMER,
};

pub fn init(apic: Apic<alloc::alloc::Global>) {
    pic8259::disable();
    if !distros_cpuid::get_feature_info().has_apic() {
        panic!("Hardware does not have APIC")
    }
    let addr = PhysAddr::new(apic.local_apic_address);
    let virt = translate_kernel(addr);
    distros_memory::map(
        PhysFrame::<Size4KiB>::containing_address(addr),
        Page::containing_address(virt),
        PageTableFlags::PRESENT
            | PageTableFlags::WRITABLE
            | PageTableFlags::NO_CACHE
            | PageTableFlags::NO_EXECUTE,
    )
    .unwrap();
    lapic::init_lapic(virt);
}
