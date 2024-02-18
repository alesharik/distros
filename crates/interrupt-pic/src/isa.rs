use crate::{Irq, IrqId};
use acpi::platform::interrupt::InterruptSourceOverride;
use log::warn;

static mut OVERRIDES: [Option<Irq>; 16] = [None; 16];

#[derive(Eq, PartialEq, Debug)]
#[repr(u8)]
pub enum IsaIrq {
    /// Programmable Interrupt Timer Interrupt
    Pit = 0,
    /// Keyboard Interrupt
    PS2Keyboard = 1,
    /// COM2 (if enabled)
    Com2 = 3,
    /// COM1 (if enabled)
    Com1 = 4,
    /// LPT2 (if enabled)
    Lpt2 = 5,
    /// Floppy Disk
    FloppyDisk = 6,
    /// LPT1 / Unreliable "spurious" interrupt (usually)
    Lpt1 = 7,
    /// CMOS real-time clock (if enabled)
    Rtc = 8,
    /// Free for peripherals / legacy SCSI / NIC
    Peripheral1 = 9,
    /// Free for peripherals / SCSI / NIC
    Peripheral2 = 10,
    /// Free for peripherals / SCSI / NIC
    Peripheral3 = 11,
    /// PS2 Mouse
    PS2Mouse = 12,
    /// FPU / Coprocessor / Inter-processor
    FPU = 13,
    /// Primary ATA Hard Disk
    PrimaryATADisk = 14,
    /// Secondary ATA Hard Disk
    SecondaryATADisk = 15,
}

impl Into<Irq> for IsaIrq {
    fn into(self) -> Irq {
        let idx = self as usize;
        unsafe {
            if let Some(ov) = OVERRIDES[idx] {
                return ov;
            }
        }
        Irq::new(IrqId::new(idx as u32))
    }
}

pub fn setup_overrides(overrides: &[InterruptSourceOverride]) {
    for x in overrides {
        if x.isa_source > 16 {
            warn!("Skipping override {:?}: isa source > 16", x);
            continue;
        }
        unsafe {
            OVERRIDES[x.isa_source as usize] = Some(
                Irq::new(IrqId::new(x.global_system_interrupt))
                    .trigger_mode(x.trigger_mode)
                    .polarity(x.polarity),
            )
        }
    }
}
