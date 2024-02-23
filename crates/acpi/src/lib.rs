#![no_std]
#![feature(allocator_api)]

#[macro_use]
extern crate alloc;

use acpi::platform::interrupt::Apic;
use acpi::{
    AcpiError, AcpiHandler, AcpiTables, AmlTable, HpetInfo, InterruptModel, PciConfigRegions,
    PhysicalMapping, SsdtIterator,
};
use core::ptr::NonNull;
use log::info;
use x86_64::PhysAddr;

#[derive(Clone)]
pub struct AcpiMemHandler;

impl AcpiHandler for AcpiMemHandler {
    unsafe fn map_physical_region<T>(
        &self,
        physical_address: usize,
        size: usize,
    ) -> PhysicalMapping<Self, T> {
        let addr = PhysAddr::new(physical_address as u64);
        PhysicalMapping::new(
            physical_address,
            NonNull::new_unchecked(distros_memory::translate_kernel(addr).as_mut_ptr()),
            size,
            size,
            self.clone(),
        )
    }

    fn unmap_physical_region<T>(_region: &PhysicalMapping<Self, T>) {}
}

static mut APIC: Option<Apic<'static, alloc::alloc::Global>> = None;
static mut HPET: Option<HpetInfo> = None;
static mut PCI_CONFIG_REGIONS: Option<PciConfigRegions<'static, alloc::alloc::Global>> = None;
static mut TABLES: Option<AcpiTables<AcpiMemHandler>> = None;

pub fn apic() -> &'static Apic<'static, alloc::alloc::Global> {
    unsafe { APIC.as_ref().expect("ACPI not initialized") }
}

pub fn hpet() -> Option<&'static HpetInfo> {
    unsafe { HPET.as_ref() }
}

pub fn pci_config_regions() -> Option<&'static PciConfigRegions<'static, alloc::alloc::Global>> {
    unsafe { PCI_CONFIG_REGIONS.as_ref() }
}

pub fn init_acpi(rdsp_addr: Option<u64>) {
    unsafe {
        TABLES = Some(
            rdsp_addr
                .map(|addr| AcpiTables::from_rsdp(AcpiMemHandler, addr as usize))
                .unwrap_or_else(|| AcpiTables::search_for_rsdp_bios(AcpiMemHandler))
                .expect("Failed to get ACPI tables"),
        );
        info!("Got ACPI tables");
        let tables = TABLES.as_ref().unwrap();
        let platform_info = tables.platform_info().expect("Failed to get platform info");
        PCI_CONFIG_REGIONS = PciConfigRegions::new(tables).ok();
        HPET = match HpetInfo::new(tables) {
            Ok(r) => Some(r),
            Err(e) => match e {
                AcpiError::TableMissing(_) => None,
                _ => panic!("{:?}", e),
            },
        };
        match platform_info.interrupt_model {
            InterruptModel::Unknown => panic!("This kernel requires APIC to run"),
            InterruptModel::Apic(apic) => APIC = Some(apic),
            _ => panic!("ACPI does not have interrupt model info"),
        }
    }
}

pub fn parse_dsdt() -> Result<AmlTable, AcpiError> {
    unsafe { TABLES.as_ref().expect("ACPI not initialized").dsdt() }
}

pub fn parse_ssdts<'a>() -> SsdtIterator<'a, AcpiMemHandler> {
    unsafe { TABLES.as_ref().expect("ACPI not initialized").ssdts() }
}
