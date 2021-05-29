use x2apic::lapic::{LocalApicBuilder, TimerDivide, LocalApic};
use crate::interrupts::{INT_LAPIC_TIMER, INT_LAPIC_ERROR, INT_LAPIC_SUPROUS};
use x86_64::VirtAddr;
use spin::Mutex;

lazy_static!(
    static ref LAPIC: Mutex<Option<LocalApic>> = Mutex::new(Option::None);
);

pub fn init_lapic(address: VirtAddr) {
    unsafe {
        let mut apic = LocalApicBuilder::new()
            .timer_vector(INT_LAPIC_TIMER)
            .error_vector(INT_LAPIC_ERROR)
            .spurious_vector(INT_LAPIC_SUPROUS)
            .set_xapic_base(address.as_u64())
            .timer_divide(TimerDivide::Div2)
            .timer_initial(100000)
            .build()
            .expect("Failed to get Local APIC");
        apic.enable();
        apic.disable_timer(); // FIXME
        let mut lapic_ref = LAPIC.lock();
        *lapic_ref = Some(apic);
        kblog!("LAPIC", "LAPIC enabled");
    }
}

pub fn eoi() {
    let mut guard = LAPIC.lock();
    let lapic = guard.as_mut().expect("Local APIC is not initialized");
    unsafe { lapic.end_of_interrupt(); }
}