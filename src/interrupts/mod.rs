macro_rules! int_handler {
    (pub noint $name:ident $body:expr) => {
        pub extern "x86-interrupt" fn $name(stack_frame: InterruptStackFrame) {
            crate::interrupts::no_int(|| {
                $body(stack_frame);
                crate::interrupts::eoi();
            })
        }
    };
    (noint $name:ident $body:expr) => {
        extern "x86-interrupt" fn $name(stack_frame: InterruptStackFrame) {
            crate::interrupts::no_int(|| {
                $body(stack_frame);
                crate::interrupts::eoi();
            })
        }
    };
    ($name:ident $body:expr) => {
        extern "x86-interrupt" fn $name(stack_frame: InterruptStackFrame) {
            $body(stack_frame)
        }
    };
}

mod pic;
mod syscall;
mod timer;

use crate::acpi::AcpiInfo;
use distros_interrupt::InterruptId;
pub use pic::{eoi, no_int};
pub use syscall::init as syscall_init;
pub use timer::now;

pub const INT_LAPIC_TIMER: InterruptId = InterruptId::new(33);
pub const INT_LAPIC_ERROR: InterruptId = InterruptId::new(34);
pub const INT_LAPIC_SUPROUS: InterruptId = InterruptId::new(35);
pub const INT_IOAPIC_OFFSET: usize = 45;
pub const RTC_IRQ: Irq = Irq::from_raw(8);

/// Representation of APIC IRQ with transparent ISR support
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct Irq(u8);

impl Irq {
    pub const fn from_raw(irq: u8) -> Self {
        Irq(irq)
    }

    /// Map IRQ to processor interrupt, enabling it
    ///
    /// # Arguments
    /// * `dest` - Destination CPU
    pub fn map_to_int(&self, dest: u32) -> InterruptId {
        InterruptId::new(pic::map_irc_irq(self.0, dest))
    }

    /// Is this IRQ already bound to handler?
    pub fn has_handler(&self) -> bool {
        if let Some(int) = pic::convert_isr_irq(self.0) {
            distros_interrupt::has_handler(InterruptId::new(int))
        } else {
            false
        }
    }
}

pub fn init_pic(acpi: &AcpiInfo) {
    pic::disable_interrupts();

    pic::init_pic(&acpi.apic);
    timer::init_timer(&acpi);

    pic::enable_interrupts();
}
