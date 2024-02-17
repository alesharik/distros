use acpi::platform::interrupt::Apic;
use acpi::{AcpiError, PciConfigRegions};
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
            NonNull::new_unchecked(crate::memory::map_physical_address(addr).as_mut_ptr()),
            size,
            size,
            self.clone(),
        )
    }

    fn unmap_physical_region<T>(_region: &PhysicalMapping<Self, T>) {}
}

#[derive(Debug)]
pub struct AcpiInfo {
    pub apic: Apic,
    pub hpet: Option<HpetInfo>,
    pub pci_config_regions: Option<PciConfigRegions>
}

pub fn init_acpi(rdsp_addr: Option<u64>) -> AcpiInfo {
    unsafe {
        let tables = rdsp_addr
            .map(|addr| AcpiTables::from_rsdp(AcpiMemHandler::new(), addr as usize))
            .unwrap_or_else(|| AcpiTables::search_for_rsdp_bios(AcpiMemHandler::new()))
            .expect("Failed to get ACPI tables");
        info!("Got ACPI tables");
        let platform_info = tables.platform_info().expect("Failed to get platform info");
        let pci_config_regions = PciConfigRegions::new(&tables).ok();
        let hpet = match HpetInfo::new(&tables) {
            Ok(r) => Some(r),
            Err(e) => match e {
                AcpiError::TableMissing(_) => None,
                _ => panic!("{:?}", e),
            },
        };
        match platform_info.interrupt_model {
            InterruptModel::Unknown => panic!("This kernel requires APIC to run"),
            InterruptModel::Apic(apic) => AcpiInfo { apic, hpet, pci_config_regions },
            _ => panic!("ACPI does not have interrupt model info"),
        }
    }
}
