macro_rules! int_handler {
    (pub noint $name:ident $body:expr) => {
        pub extern "x86-interrupt" fn $name(stack_frame: InterruptStackFrame)  {
            crate::interrupts::no_int(|| {
                $body(stack_frame);
                crate::interrupts::eoi();
            })
        }
    };
    (noint $name:ident $body:expr) => {
        extern "x86-interrupt" fn $name(stack_frame: InterruptStackFrame)  {
            crate::interrupts::no_int(|| {
                $body(stack_frame);
                crate::interrupts::eoi();
            })
        }
    };
    ($name:ident $body:expr) => {
        extern "x86-interrupt" fn $name(stack_frame: InterruptStackFrame)  {
            $body(stack_frame)
        }
    };
}

mod idt;
mod pic;
mod timer;

use crate::acpi::AcpiInfo;

pub use idt::{init_idt, set_handler};
pub use timer::{now, sleep};
pub use pic::{eoi, no_int};

pub const INT_LAPIC_TIMER: InterruptId = InterruptId::from_raw(33);
pub const INT_LAPIC_ERROR: InterruptId = InterruptId::from_raw(34);
pub const INT_LAPIC_SUPROUS: InterruptId = InterruptId::from_raw(35);
pub const INT_IOAPIC_OFFSET: usize = 45;
pub const RTC_IRQ: Irq = Irq::from_raw(8);

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct InterruptId(usize);

impl InterruptId {
    const fn from_raw(int: usize) -> Self {
        InterruptId(int)
    }
}

/// Representation of APIC IRQ with transparent ISR support
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct Irq(u8);

impl Irq {
    const fn from_raw(irq: u8) -> Self {
        Irq(irq)
    }

    /// Map IRQ to processor interrupt, enabling it
    ///
    /// # Arguments
    /// * `dest` - Destination CPU
    fn map_to_int(&self, dest: u32) -> InterruptId {
        InterruptId::from_raw(pic::map_irc_irq(self.0, dest))
    }

    /// Is this IRQ already bound to handler?
    fn has_handler(&self) -> bool {
        idt::has_int_handler(InterruptId::from_raw(INT_IOAPIC_OFFSET + self.0 as usize))
    }
}

pub fn init_pic(acpi: &AcpiInfo) {
    pic::disable_interrupts();

    pic::init_pic(&acpi.apic);
    timer::init_timer(&acpi);

    pic::enable_interrupts();
}