use crate::{Irq, IrqDestination, IrqId, IrqMode};
use acpi::platform::interrupt::{IoApic, Polarity, TriggerMode};
use alloc::vec::Vec;
use distros_interrupt::InterruptId;
use distros_memory::translate_kernel;
use log::info;
use spin::Mutex;
use x2apic::ioapic::IrqFlags;
use x86_64::structures::paging::{Page, PageTableFlags, PhysFrame, Size4KiB};
use x86_64::PhysAddr;

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum IoApicError {
    IoApicNotFound { gsi: IrqId },
}

struct IoApicHolder {
    gsi_start: u32,
    gsi_end: u32,
    ioapic: Mutex<x2apic::ioapic::IoApic>,
}

impl IoApicHolder {
    pub fn contains(&self, irq: IrqId) -> bool {
        let gsi = irq.id();
        gsi >= self.gsi_start && gsi <= self.gsi_end
    }
}

static mut HOLDERS: Option<Vec<IoApicHolder>> = None;

pub fn init(apics: &[IoApic]) {
    let mut holders = Vec::new();
    unsafe {
        for apic in apics {
            let addr = PhysAddr::new(apic.address as u64);
            let virt_addr = translate_kernel(addr);
            distros_memory::map(
                PhysFrame::<Size4KiB>::containing_address(addr),
                Page::containing_address(virt_addr),
                PageTableFlags::PRESENT
                    | PageTableFlags::WRITABLE
                    | PageTableFlags::NO_CACHE
                    | PageTableFlags::NO_EXECUTE,
            )
            .unwrap();
            let mut ioapic = x2apic::ioapic::IoApic::new(virt_addr.as_u64());
            info!(
                "IOAPIC {} initialized: GSI {}..{}",
                apic.id,
                apic.global_system_interrupt_base,
                apic.global_system_interrupt_base + ioapic.max_table_entry() as u32
            );
            holders.push(IoApicHolder {
                gsi_start: apic.global_system_interrupt_base,
                gsi_end: apic.global_system_interrupt_base + ioapic.max_table_entry() as u32,
                ioapic: Mutex::new(ioapic),
            });
        }
        HOLDERS = Some(holders)
    }
}

/// **Note: disables IRQ before modifications**
pub fn set_entry(
    irq: Irq,
    dest: IrqDestination,
    vector: InterruptId,
    mode: IrqMode,
) -> Result<(), IoApicError> {
    let apics = unsafe { HOLDERS.as_ref().expect("IOAPIC not initialized") };
    apics
        .iter()
        .find(|a| a.contains(irq.get_global_system_interrupt()))
        .ok_or(IoApicError::IoApicNotFound {
            gsi: irq.get_global_system_interrupt(),
        })
        .map(|a| {
            let mut apic = a.ioapic.lock();
            let i = (irq.get_global_system_interrupt().id() - a.gsi_start) as u8;
            unsafe {
                apic.disable_irq(i);
                let mut entry = apic.table_entry(i);
                entry.set_vector(vector.int() as u8);
                entry.set_dest(dest.into());
                entry.set_mode(match mode {
                    IrqMode::Fixed => x2apic::ioapic::IrqMode::Fixed,
                    IrqMode::LowestPriority => x2apic::ioapic::IrqMode::LowestPriority,
                    IrqMode::SystemManagement => x2apic::ioapic::IrqMode::SystemManagement,
                    IrqMode::NonMaskable => x2apic::ioapic::IrqMode::NonMaskable,
                    IrqMode::Init => x2apic::ioapic::IrqMode::Init,
                    IrqMode::External => x2apic::ioapic::IrqMode::External,
                });
                let mut flags = entry.flags();
                match irq.get_polarity() {
                    Polarity::SameAsBus => {}
                    Polarity::ActiveHigh => flags.set(IrqFlags::LOW_ACTIVE, false),
                    Polarity::ActiveLow => flags.set(IrqFlags::LOW_ACTIVE, true),
                }
                match irq.get_trigger_mode() {
                    TriggerMode::SameAsBus => {}
                    TriggerMode::Edge => flags.set(IrqFlags::LEVEL_TRIGGERED, false),
                    TriggerMode::Level => flags.set(IrqFlags::LEVEL_TRIGGERED, true),
                }
                entry.set_flags(flags);
                apic.set_table_entry(i, entry);
            }
        })
}

pub fn enable(irq: IrqId) -> Result<(), IoApicError> {
    let apics = unsafe { HOLDERS.as_ref().expect("IOAPIC not initialized") };
    apics
        .iter()
        .find(|a| a.contains(irq))
        .ok_or(IoApicError::IoApicNotFound { gsi: irq })
        .map(|a| {
            let mut apic = a.ioapic.lock();
            unsafe {
                apic.enable_irq((irq.id() - a.gsi_start) as u8);
            }
        })
}

pub fn disable(irq: IrqId) -> Result<(), IoApicError> {
    let apics = unsafe { HOLDERS.as_ref().expect("IOAPIC not initialized") };
    apics
        .iter()
        .find(|a| a.contains(irq))
        .ok_or(IoApicError::IoApicNotFound { gsi: irq })
        .map(|a| {
            let mut apic = a.ioapic.lock();
            unsafe {
                apic.disable_irq((irq.id() - a.gsi_start) as u8);
            }
        })
}
