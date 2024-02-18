macro_rules! int_handler {
    (pub noint $name:ident $body:expr) => {
        pub extern "x86-interrupt" fn $name(stack_frame: InterruptStackFrame) {
            x86_64::instructions::interrupts::without_interrupts(|| {
                $body(stack_frame);
                distros_interrupt_pic::lapic_eoi();
            })
        }
    };
    (noint $name:ident $body:expr) => {
        extern "x86-interrupt" fn $name(stack_frame: InterruptStackFrame) {
            x86_64::instructions::interrupts::without_interrupts(|| {
                $body(stack_frame);
                distros_interrupt_pic::lapic_eoi();
            })
        }
    };
    ($name:ident $body:expr) => {
        extern "x86-interrupt" fn $name(stack_frame: InterruptStackFrame) {
            $body(stack_frame)
        }
    };
}

mod syscall;
pub mod timer;

use distros_interrupt::InterruptId;
use x86_64::instructions::interrupts;

pub const INT_LAPIC_TIMER: InterruptId = InterruptId::new(33);
pub const INT_LAPIC_ERROR: InterruptId = InterruptId::new(34);
pub const INT_LAPIC_SUPROUS: InterruptId = InterruptId::new(35);
pub const INT_IOAPIC_OFFSET: usize = 45;

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
        unimplemented!();
        // InterruptId::new(pic::map_irc_irq(self.0, dest))
    }

    /// Is this IRQ already bound to handler?
    pub fn has_handler(&self) -> bool {
        unimplemented!();
        // if let Some(int) = pic::convert_isr_irq(self.0) {
        //     distros_interrupt::has_handler(InterruptId::new(int))
        // } else {
        //     false
        // }
    }
}

pub fn init_pic() {
    timer::init_timer();
}
