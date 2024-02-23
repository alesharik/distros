use distros_interrupt::OverrideMode;
use distros_interrupt::{int_handler, InterruptId};
use log::{error, info};
use x2apic::lapic::{LocalApic, LocalApicBuilder, TimerDivide, TimerMode};
use x86_64::registers::model_specific::Msr;
use x86_64::structures::idt::InterruptStackFrame;
use x86_64::{software_interrupt, VirtAddr};

pub const INT_LAPIC_TIMER: InterruptId = InterruptId::new(32);
const INT_LAPIC_ERROR: InterruptId = InterruptId::new(0xFF - 1);
const INT_LAPIC_SPURIOUS: InterruptId = InterruptId::new(0xFF);

static mut LAPIC: Option<LocalApic> = None;
const IA32_TSC_DEADLINE_MSR: Msr = Msr::new(0x6E0);

pub fn init_lapic(address: VirtAddr) {
    unsafe {
        distros_interrupt::set_handler(INT_LAPIC_ERROR, lapic_error, OverrideMode::Panic);
        distros_interrupt::set_handler(INT_LAPIC_SPURIOUS, lapic_suprous, OverrideMode::Panic);
        let mut apic = LocalApicBuilder::new()
            .timer_vector(INT_LAPIC_TIMER.int())
            .error_vector(INT_LAPIC_ERROR.int())
            .spurious_vector(INT_LAPIC_SPURIOUS.int())
            .set_xapic_base(address.as_u64())
            .build()
            .expect("Failed to get Local APIC");
        apic.enable();
        apic.disable_timer();
        info!("LAPIC {} at 0x{:08x} enabled", apic.id(), address);
        LAPIC = Some(apic);
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

pub fn timer_set_mode(mode: TimerMode, timer_divide: TimerDivide) {
    unsafe {
        let lapic = LAPIC.as_mut().expect("Local APIC is not initialized");
        lapic.set_timer_mode(mode);
        lapic.set_timer_divide(timer_divide);
    }
}

pub fn timer_add_initial(initial: u32) {
    unsafe {
        let lapic = LAPIC.as_mut().expect("Local APIC is not initialized");
        lapic.set_timer_initial(
            initial
                .checked_add(lapic.timer_current())
                .unwrap_or_else(|| lapic.timer_current() - initial),
        );
    }
}

pub fn timer_set_tsc_deadline(deadline: u64) {
    unsafe {
        IA32_TSC_DEADLINE_MSR.write(deadline);
    }
}

pub fn timer_enable() {
    unsafe {
        let lapic = LAPIC.as_mut().expect("Local APIC is not initialized");
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
        };
        error!("EXCEPTION: LAPIC ERROR {:?}\n{:#?}", flags, stack_frame);
    }
);

int_handler!(
    lapic_suprous | stack_frame: InterruptStackFrame | {
        error!("EXCEPTION: LAPIC SUPROUS\n{:#?}", stack_frame);
    }
);
