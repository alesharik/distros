use crate::interrupts::{INT_LAPIC_ERROR, INT_LAPIC_SUPROUS, INT_LAPIC_TIMER};
use x2apic::lapic::{LocalApic, LocalApicBuilder, TimerDivide};
use x86_64::{software_interrupt, VirtAddr};

static mut LAPIC: Option<LocalApic> = None;

pub fn init_lapic(address: VirtAddr) {
    unsafe {
        let mut apic = LocalApicBuilder::new()
            .timer_vector(INT_LAPIC_TIMER.int())
            .error_vector(INT_LAPIC_ERROR.int())
            .spurious_vector(INT_LAPIC_SUPROUS.int())
            .set_xapic_base(address.as_u64())
            .timer_divide(TimerDivide::Div4)
            .timer_initial(5_000_000)
            .build()
            .expect("Failed to get Local APIC");
        apic.enable();
        LAPIC = Some(apic);
        info!("LAPIC enabled");
    }
}

pub fn eoi() {
    unsafe {
        LAPIC
            .as_mut()
            .expect("Local APIC is not initialized")
            .end_of_interrupt();
    }
}

pub fn start_lapic_timer() {
    unsafe {
        LAPIC
            .as_mut()
            .expect("Local APIC is not initialized")
            .enable_timer();
    }
}

pub unsafe fn invoke_lapic_timer_interrupt() {
    use core::arch::asm;
    software_interrupt!(INT_LAPIC_TIMER.int());
}
