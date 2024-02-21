use acpi::mcfg::PciConfigEntry;
use acpi::PciConfigRegions;
use alloc::collections::BTreeMap;
use alloc::sync::Arc;
use alloc::vec::Vec;
use distros_memory::translate_kernel;
use log::debug;
use pci_types::{ConfigRegionAccess, PciAddress};
use x86_64::structures::paging::{Page, PageTableFlags, PhysFrame, Size4KiB};
use x86_64::{PhysAddr, VirtAddr};

const ADDR_OFFSET: VirtAddr = VirtAddr::new_truncate(1024 * 1024 * 1024 * 512);

pub struct PcieAccess {
    regions: Vec<PciConfigEntry>,
}

impl PcieAccess {
    pub fn new(regions: &PciConfigRegions<'_, alloc::alloc::Global>) -> Self {
        for x in regions.iter() {
            let addr = PhysAddr::new(x.physical_address as u64);
            let virt = ADDR_OFFSET + addr.as_u64();
            distros_memory::map(
                PhysFrame::<Size4KiB>::containing_address(addr),
                Page::containing_address(virt),
                PageTableFlags::PRESENT
                    | PageTableFlags::WRITABLE
                    | PageTableFlags::NO_CACHE
                    | PageTableFlags::NO_EXECUTE,
            )
            .unwrap();
            debug!("Mapped PCIe addr {:?} to {:?}", addr, virt);
        }
        PcieAccess {
            regions: regions.iter().collect(),
        }
    }

    fn get_address(&self, address: PciAddress) -> Option<VirtAddr> {
        let region = self.regions.iter().find(|region| {
            region.segment_group == address.segment() && region.bus_range.contains(&address.bus())
        })?;

        Some(translate_kernel(
            PhysAddr::new(region.physical_address as u64)
                + ((u64::from(address.bus() - region.bus_range.start()) << 20)
                    | (u64::from(address.device()) << 15)
                    | (u64::from(address.function()) << 12)),
        ))
    }
}

impl ConfigRegionAccess for PcieAccess {
    fn function_exists(&self, address: PciAddress) -> bool {
        unsafe { self.read(address, 0) & 0xFFFF != 0xFFFF }
    }

    unsafe fn read(&self, address: PciAddress, offset: u16) -> u32 {
        if let Some(mem) = self.get_address(address) {
            (mem + offset as u64).as_ptr::<u32>().read_volatile()
        } else {
            u32::MAX
        }
    }

    unsafe fn write(&self, address: PciAddress, offset: u16, value: u32) {
        if let Some(mem) = self.get_address(address) {
            (mem + offset as u64)
                .as_mut_ptr::<u32>()
                .write_volatile(value);
        }
    }
}
