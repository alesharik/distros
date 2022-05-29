use alloc::sync::Arc;
use bit_field::BitField;
use pci_types::{ConfigRegionAccess, PciAddress};
use spin::Mutex;
use x86_64::instructions::port::{Port, PortWriteOnly};

struct PciAccessInner {
    address_port: PortWriteOnly<u32>,
    data_port: Port<u32>,
}


#[derive(Clone)]
pub struct PciAccess {
    inner: Arc<Mutex<PciAccessInner>>
}

impl PciAccess {
    pub fn new() -> Self {
        PciAccess {
            inner: Arc::new(Mutex::new(PciAccessInner {
                address_port: PortWriteOnly::new(0xCF8),
                data_port: Port::new(0xCFC),
            }))
        }
    }
}

impl ConfigRegionAccess for PciAccess {
    fn function_exists(&self, address: PciAddress) -> bool {
        unsafe { self.read(address, 0) & 0xFFFF != 0xFFFF }
    }

    unsafe fn read(&self, address: PciAddress, offset: u16) -> u32 {
        crate::interrupts::no_int(|| {
            let mut result: u32 = 0;
            result.set_bits(0..8, offset as u32);
            result.set_bits(8..11, address.function() as u32);
            result.set_bits(11..16, address.device() as u32);
            result.set_bits(16..23, address.bus() as u32);
            result.set_bit(31, true);
            let mut inner = self.inner.lock();
            inner.address_port.write(result);
            inner.data_port.read()
        })
    }

    unsafe fn write(&self, address: PciAddress, offset: u16, value: u32) {
        crate::interrupts::no_int(|| {
            let mut result = 0u32;
            result.set_bits(0..8, offset as u32);
            result.set_bits(8..11, address.function() as u32);
            result.set_bits(11..16, address.device() as u32);
            result.set_bits(16..23, address.bus() as u32);
            result.set_bit(31, true);
            let mut inner = self.inner.lock();
            inner.address_port.write(result);
            inner.data_port.write(value);
        })
    }
}
