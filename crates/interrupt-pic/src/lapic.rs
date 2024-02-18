use distros_interrupt::idt::OverrideMode;
use distros_interrupt::InterruptId;
use log::{error, info};
use x2apic::lapic::{LocalApic, LocalApicBuilder, TimerDivide, TimerMode};
use x86_64::structures::idt::InterruptStackFrame;
use x86_64::{software_interrupt, VirtAddr};

pub const INT_LAPIC_TIMER: InterruptId = InterruptId::new(33);
const INT_LAPIC_ERROR: InterruptId = InterruptId::new(0xFF - 1);
const INT_LAPIC_SPURIOUS: InterruptId = InterruptId::new(0xFF);

static mut LAPIC: Option<LocalApic> = None;

pub fn init_lapic(address: VirtAddr) {
    unsafe {
        let mut apic = LocalApicBuilder::new()
            .timer_vector(INT_LAPIC_TIMER.0)
            .error_vector(INT_LAPIC_ERROR.0)
            .spurious_vector(INT_LAPIC_SPURIOUS.0)
            .set_xapic_base(address.as_u64())
            .build()
            .expect("Failed to get Local APIC");
        apic.enable();
        LAPIC = Some(apic);
        distros_interrupt::idt::set_handler(INT_LAPIC_ERROR, lapic_error, OverrideMode::Panic);
        distros_interrupt::idt::set_handler(INT_LAPIC_SPURIOUS, lapic_suprous, OverrideMode::Panic);
        info!("LAPIC enabled");
    }
}

pub fn eoi() {
    unsafe {
        LAPIC
            .as_mut()
            .expect("Local APIC is not initialized")
            .error_flags()
            .end_of_interrupt();
    }
}

pub fn timer_enable(mode: TimerMode, divide: TimerDivide, initial: u32) {
    unsafe {
        let lapic = LAPIC.as_mut().expect("Local APIC is not initialized");
        lapic.set_timer_mode(mode);
        lapic.set_timer_divide(divide);
        lapic.set_timer_initial(initial);
        lapic.enable_timer();
    }
}

pub fn timer_disable() {
    unsafe {
        LAPIC
            .as_mut()
            .expect("Local APIC is not initialized")
            .disable_timer();
    }
}

int_handler!(
    lapic_error | stack_frame: InterruptStackFrame | {
        let flags = unsafe {
            LAPIC
                .as_mut()
                .expect("Local APIC is not initialized")
                .error_flags()
        }
        error!("EXCEPTION: LAPIC ERROR {:?}\n{:#?}", flags, stack_frame);
    }
);

int_handler!(
    lapic_suprous | stack_frame: InterruptStackFrame | {
        error!("EXCEPTION: LAPIC SUPROUS\n{:#?}", stack_frame);
    }
);
