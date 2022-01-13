use crate::kblog;
use acpi::platform::interrupt::Apic;
use acpi::AcpiError;
use acpi::InterruptModel;
use acpi::{AcpiHandler, AcpiTables, HpetInfo, PhysicalMapping};
use core::ptr::NonNull;
use x86_64::PhysAddr;

#[derive(Clone)]
struct AcpiMemHandler {}

impl AcpiMemHandler {
    fn new() -> Self {
        AcpiMemHandler {}
    }
}

impl AcpiHandler for AcpiMemHandler {
    unsafe fn map_physical_region<T>(
        &self,
        physical_address: usize,
        size: usize,
    ) -> PhysicalMapping<Self, T> {
        let addr = PhysAddr::new(physical_address as u64);
        PhysicalMapping::new(
            physical_address,
            NonNull::new_unchecked(
                crate::memory::map_physical_address(addr).as_mut_ptr(),
            ),
            size,
            size,
            self.clone()
        )
    }

    fn unmap_physical_region<T>(_region: &PhysicalMapping<Self, T>) {}
}

#[derive(Debug)]
pub struct AcpiInfo {
    pub apic: Apic,
    pub hpet: Option<HpetInfo>,
}

pub fn init_acpi() -> AcpiInfo {
    unsafe {
        let tables = AcpiTables::search_for_rsdp_bios(AcpiMemHandler::new())
            .expect("Failed to get ACPI tables");
        kblog!("ACPI", "Got ACPi tables");
        let platform_info = tables.platform_info().expect("Failed to get platform info");
        let hpet = match HpetInfo::new(&tables) {
            Ok(r) => Some(r),
            Err(e) => match e {
                AcpiError::TableMissing(_) => None,
                _ => panic!("{:?}", e),
            },
        };
        match platform_info.interrupt_model {
            InterruptModel::Unknown => panic!("This kernel requires APIC to run"),
            InterruptModel::Apic(apic) => AcpiInfo { apic, hpet },
            _ => panic!("ACPI does not have interrupt model info"),
        }
    }
}
