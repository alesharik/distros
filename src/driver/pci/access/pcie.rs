use acpi::PciConfigRegions;
use alloc::sync::Arc;
use distros_memory::translate_kernel;
use pci_types::{ConfigRegionAccess, PciAddress};
use x86_64::PhysAddr;

#[derive(Clone)]
pub struct PcieAccess {
    // regions: Arc<PciConfigRegions<alloc::alloc::Global>>,
}

impl PcieAccess {
    pub fn new(regions: &PciConfigRegions<'_, alloc::alloc::Global>) -> Self {
        // Self {
        //     regions: Arc::new(regions.clone()),
        // }
        unimplemented!()
    }
}

impl ConfigRegionAccess for PcieAccess {
    fn function_exists(&self, address: PciAddress) -> bool {
        unsafe { self.read(address, 0) & 0xFFFF != 0xFFFF }
    }

    unsafe fn read(&self, address: PciAddress, offset: u16) -> u32 {
        // if let Some(mem) = self.regions.physical_address(
        //     address.segment(),
        //     address.bus(),
        //     address.device(),
        //     address.function(),
        // ) {
        //     let phys = PhysAddr::new(mem + offset as u64);
        //     let virt = translate_kernel(phys);
        //     *(virt.as_ptr())
        // } else {
        //     u32::MAX
        // }
        unimplemented!()
    }

    unsafe fn write(&self, address: PciAddress, offset: u16, value: u32) {
        // if let Some(mem) = self.regions.physical_address(
        //     address.segment(),
        //     address.bus(),
        //     address.device(),
        //     address.function(),
        // ) {
        //     let phys = PhysAddr::new(mem + offset as u64);
        //     let virt = translate_kernel(phys);
        //     *(virt.as_mut_ptr()) = value;
        // }
        unimplemented!()
    }
}
