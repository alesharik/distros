use alloc::sync::Arc;
use acpi::PciConfigRegions;
use pci_types::{ConfigRegionAccess, PciAddress};
use x86_64::PhysAddr;
use distros_memory::translate_kernel;

#[derive(Clone)]
pub struct PcieAccess {
    regions: Arc<PciConfigRegions>
}

impl PcieAccess {
    pub fn new(regions: &PciConfigRegions) -> Self {
        Self {
            regions: Arc::new(regions.clone())
        }
    }
}

impl ConfigRegionAccess for PcieAccess {
    fn function_exists(&self, address: PciAddress) -> bool {
        unsafe { self.read(address, 0) & 0xFFFF != 0xFFFF }
    }

    unsafe fn read(&self, address: PciAddress, offset: u16) -> u32 {
        if let Some(mem) = self.regions.physical_address(address.segment(), address.bus(), address.device(), address.function()) {
            let phys = PhysAddr::new(mem + offset as u64);
            let virt = translate_kernel(phys);
            *(virt.as_ptr())
        } else {
            u32::MAX
        }
    }

    unsafe fn write(&self, address: PciAddress, offset: u16, value: u32) {
        if let Some(mem) = self.regions.physical_address(address.segment(), address.bus(), address.device(), address.function()) {
            let phys = PhysAddr::new(mem + offset as u64);
            let virt = translate_kernel(phys);
            *(virt.as_mut_ptr()) = value;
        }
    }
}
