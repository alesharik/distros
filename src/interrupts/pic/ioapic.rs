use acpi::platform::Apic;
use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use spin::Mutex;
use x2apic::ioapic::{IoApic, IrqFlags, IrqMode};
use x86_64::PhysAddr;

struct IoApicHolder {
    id: u8,
    global_system_interrupt_base: u32,
    handler: IoApic,
}

impl IoApicHolder {
    fn in_range(&self, irq: u32) -> bool {
        irq >= self.global_system_interrupt_base && irq < (self.global_system_interrupt_base + 24)
    }
}

struct IoApicManager {
    apics: Vec<IoApicHolder>,
    override_map: BTreeMap<u8, u32>,
}

impl IoApicManager {
    fn new(apic: &Apic) -> Self {
        unsafe {
            let mut apics = Vec::<IoApicHolder>::new();
            for io_apic in &apic.io_apics {
                let addr = PhysAddr::new(apic.io_apics[0].address as u64);
                let virt_addr = crate::memory::map_physical_address(addr);
                apics.push(IoApicHolder {
                    id: io_apic.id,
                    global_system_interrupt_base: io_apic.global_system_interrupt_base,
                    handler: IoApic::new(virt_addr.as_u64()),
                })
            }
            let mut override_map = BTreeMap::<u8, u32>::new();
            for int in &apic.interrupt_source_overrides {
                override_map.insert(int.isa_source, int.global_system_interrupt);
            }
            IoApicManager {
                apics,
                override_map,
            }
        }
    }

    fn init(&mut self) {
        for apic in self.apics.iter_mut() {
            unsafe {
                apic.handler.init(
                    (crate::interrupts::INT_IOAPIC_OFFSET
                        + apic.global_system_interrupt_base as usize) as u8,
                )
            }
        }
    }

    fn map_isr_irq(&mut self, isr: u8, dest: u32) -> usize {
        let isr_u32 = isr as u32;
        let global_irq = self.override_map.get(&isr).unwrap_or(&(isr_u32));
        let apic = self
            .apics
            .iter_mut()
            .filter(|holder| holder.in_range(*global_irq))
            .next();
        if let Some(apic) = apic {
            unsafe {
                apic.handler
                    .enable_irq(*global_irq as u8, dest, IrqMode::Fixed, IrqFlags::empty())
            }
            crate::interrupts::INT_IOAPIC_OFFSET + (*global_irq as usize)
        } else {
            panic!("Could not find IOAPIC for IRQ {}", global_irq)
        }
    }

    fn convert_from_isr_to_irq(&self, isr: u8) -> Option<usize> {
        let isr_u32 = isr as u32;
        let global_irq = self.override_map.get(&isr).unwrap_or(&(isr_u32));
        let apic = self
            .apics
            .iter()
            .filter(|holder| holder.in_range(*global_irq))
            .next();
        if let Some(_) = apic {
            Some(crate::interrupts::INT_IOAPIC_OFFSET + (*global_irq as usize))
        } else {
            None
        }
    }
}

lazy_static! {
    static ref IOAPIC_MANAGER: Mutex<Option<IoApicManager>> = Mutex::new(Option::None);
}

pub fn init_ioapic(apic: &Apic) {
    let mut global_manager = IOAPIC_MANAGER.lock();
    let mut manager = IoApicManager::new(apic);
    manager.init();
    *global_manager = Option::Some(manager);
    kblog!("IOAPIC", "IOAPIC set up")
}

pub fn map_irc_irq(isr: u8, dest: u32) -> usize {
    let mut guard = IOAPIC_MANAGER.lock();
    let manager = guard.as_mut().expect("IOAPIC manager is not initialized");
    manager.map_isr_irq(isr, dest)
}

pub fn convert_isr_irq(isr: u8) -> Option<usize> {
    let mut guard = IOAPIC_MANAGER.lock();
    let manager = guard.as_mut().expect("IOAPIC manager is not initialized");
    manager.convert_from_isr_to_irq(isr)
}
