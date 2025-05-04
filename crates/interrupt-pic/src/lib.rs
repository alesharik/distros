#![no_std]
#![feature(abi_x86_interrupt)]

#[macro_use]
extern crate alloc;

use acpi::platform::interrupt::{Polarity, TriggerMode};
use x86_64::structures::paging::{Page, PageTableFlags, PhysFrame, Size4KiB};
use x86_64::{PhysAddr, VirtAddr};

mod ioapic;
mod isa;
mod lapic;
mod pic8259;

pub use ioapic::{
    disable as ioapic_disable, enable as ioapic_enable, set_entry as ioapic_set_entry,
};
pub use isa::IsaIrq;
pub use lapic::{
    eoi as lapic_eoi, timer_add_initial as lapic_timer_add_initial,
    timer_disable as lapic_timer_disable, timer_enable as lapic_timer_enable,
    timer_set_mode as lapic_timer_set_mode, timer_set_tsc_deadline as lapic_timer_set_tsc_deadline,
    INT_LAPIC_TIMER,
};

const LAPIC_ADDR: VirtAddr = VirtAddr::new_truncate(1024 * 1024 * 1024 * 500);

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
#[repr(transparent)]
pub struct IrqId(u32);

impl IrqId {
    pub const fn new(id: u32) -> IrqId {
        IrqId(id)
    }

    pub const fn id(&self) -> u32 {
        self.0
    }
}

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub struct Irq {
    global_system_interrupt: IrqId,
    polarity: Polarity,
    trigger_mode: TriggerMode,
}

impl Irq {
    pub fn new(int: IrqId) -> Irq {
        Irq {
            global_system_interrupt: int,
            polarity: Polarity::SameAsBus,
            trigger_mode: TriggerMode::SameAsBus,
        }
    }

    pub fn polarity(mut self, polarity: Polarity) -> Self {
        self.polarity = polarity;
        self
    }

    pub fn trigger_mode(mut self, mode: TriggerMode) -> Self {
        self.trigger_mode = mode;
        self
    }

    #[inline]
    pub fn get_global_system_interrupt(&self) -> IrqId {
        self.global_system_interrupt
    }

    #[inline]
    pub fn get_polarity(&self) -> Polarity {
        self.polarity
    }

    #[inline]
    pub fn get_trigger_mode(&self) -> TriggerMode {
        self.trigger_mode
    }
}

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
pub enum IrqDestination {
    Local,
}

impl Into<u8> for IrqDestination {
    fn into(self) -> u8 {
        match self {
            IrqDestination::Local => 0,
        }
    }
}

/// IOAPIC interrupt modes.
#[derive(Debug)]
#[repr(u8)]
pub enum IrqMode {
    /// Asserts the INTR signal on all allowed processors.
    Fixed = 0b000,
    /// Asserts the INTR signal on the lowest priority processor allowed.
    LowestPriority = 0b001,
    /// System management interrupt.
    /// Requires edge-triggering.
    SystemManagement = 0b010,
    /// Asserts the NMI signal on all allowed processors.
    /// Requires edge-triggering.
    NonMaskable = 0b100,
    /// Asserts the INIT signal on all allowed processors.
    /// Requires edge-triggering.
    Init = 0b101,
    /// Asserts the INTR signal as a signal that originated in an
    /// externally-connected interrupt controller.
    /// Requires edge-triggering.
    External = 0b111,
}

pub fn init() {
    pic8259::disable();
    if !distros_cpuid::get_feature_info().has_apic() {
        panic!("Hardware does not have APIC")
    }
    let apic = distros_acpi::apic();
    let addr = PhysAddr::new(apic.local_apic_address);
    distros_memory::map(
        PhysFrame::<Size4KiB>::containing_address(addr),
        Page::containing_address(LAPIC_ADDR),
        PageTableFlags::PRESENT
            | PageTableFlags::WRITABLE
            | PageTableFlags::NO_CACHE
            | PageTableFlags::NO_EXECUTE,
    )
    .unwrap();
    lapic::init_lapic(LAPIC_ADDR);
    isa::setup_overrides(&apic.interrupt_source_overrides);
    ioapic::init(&apic.io_apics)
}
