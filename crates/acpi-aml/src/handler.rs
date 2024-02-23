use aml::Handler;
use bit_field::BitField;
use log::debug;
use pci_types::{ConfigRegionAccess, PciAddress};
use x86_64::instructions::port::{PortReadOnly, PortWriteOnly};
use x86_64::PhysAddr;

pub struct HandlerImpl;

impl Handler for HandlerImpl {
    fn read_u8(&self, address: usize) -> u8 {
        debug!("READu8 {}", address);
        let addr = distros_memory::translate_kernel(PhysAddr::new(address as u64));
        unsafe { *addr.as_ptr() }
    }

    fn read_u16(&self, address: usize) -> u16 {
        debug!("READu8 {}", address);
        let addr = distros_memory::translate_kernel(PhysAddr::new(address as u64));
        unsafe { *addr.as_ptr() }
    }

    fn read_u32(&self, address: usize) -> u32 {
        debug!("READu32 {}", address);
        let addr = distros_memory::translate_kernel(PhysAddr::new(address as u64));
        unsafe { *addr.as_ptr() }
    }

    fn read_u64(&self, address: usize) -> u64 {
        debug!("READu64 {}", address);
        let addr = distros_memory::translate_kernel(PhysAddr::new(address as u64));
        unsafe { *addr.as_ptr() }
    }

    fn write_u8(&mut self, address: usize, value: u8) {
        debug!("WriteU8 {} {}", address, value);
        let addr = distros_memory::translate_kernel(PhysAddr::new(address as u64));
        unsafe { *addr.as_mut_ptr() = value }
    }

    fn write_u16(&mut self, address: usize, value: u16) {
        debug!("WriteU16 {} {}", address, value);
        let addr = distros_memory::translate_kernel(PhysAddr::new(address as u64));
        unsafe { *addr.as_mut_ptr() = value }
    }

    fn write_u32(&mut self, address: usize, value: u32) {
        debug!("WriteU32 {} {}", address, value);
        let addr = distros_memory::translate_kernel(PhysAddr::new(address as u64));
        unsafe { *addr.as_mut_ptr() = value }
    }

    fn write_u64(&mut self, address: usize, value: u64) {
        debug!("WriteU64 {} {}", address, value);
        let addr = distros_memory::translate_kernel(PhysAddr::new(address as u64));
        unsafe { *addr.as_mut_ptr() = value }
    }

    fn read_io_u8(&self, port: u16) -> u8 {
        debug!("ReadIO8 {}", port);
        unsafe { PortReadOnly::new(port).read() }
    }

    fn read_io_u16(&self, port: u16) -> u16 {
        debug!("ReadIO16 {}", port);
        unsafe { PortReadOnly::new(port).read() }
    }

    fn read_io_u32(&self, port: u16) -> u32 {
        debug!("ReadIO32 {}", port);
        unsafe { PortReadOnly::new(port).read() }
    }

    fn write_io_u8(&self, port: u16, value: u8) {
        debug!("WriteIO8 {} {}", port, value);
        unsafe { PortWriteOnly::new(port).write(value) }
    }

    fn write_io_u16(&self, port: u16, value: u16) {
        debug!("WriteIO16 {} {}", port, value);
        unsafe { PortWriteOnly::new(port).write(value) }
    }

    fn write_io_u32(&self, port: u16, value: u32) {
        debug!("WriteIO32 {} {}", port, value);
        unsafe { PortWriteOnly::new(port).write(value) }
    }

    fn read_pci_u8(&self, segment: u16, bus: u8, device: u8, function: u8, offset: u16) -> u8 {
        let address = PciAddress::new(segment, bus, device, function);
        debug!("ReadPCI8 {} {}", address, offset);
        (unsafe { distros_pci_access::access().read(address, offset) & 0xFF }) as u8
    }

    fn read_pci_u16(&self, segment: u16, bus: u8, device: u8, function: u8, offset: u16) -> u16 {
        let address = PciAddress::new(segment, bus, device, function);
        debug!("ReadPCI16 {} {}", address, offset);
        (unsafe { distros_pci_access::access().read(address, offset) & 0xFFFF }) as u16
    }

    fn read_pci_u32(&self, segment: u16, bus: u8, device: u8, function: u8, offset: u16) -> u32 {
        let address = PciAddress::new(segment, bus, device, function);
        debug!("ReadPCI32 {} {}", address, offset);
        unsafe { distros_pci_access::access().read(address, offset) }
    }

    fn write_pci_u8(
        &self,
        segment: u16,
        bus: u8,
        device: u8,
        function: u8,
        offset: u16,
        value: u8,
    ) {
        let address = PciAddress::new(segment, bus, device, function);
        debug!("WritePCI8 {} {} {}", address, offset, value);
        unsafe {
            let mut read = distros_pci_access::access().read(address, offset);
            read.set_bits(0..8, value as u32);
            distros_pci_access::access().write(address, offset, read);
        }
    }

    fn write_pci_u16(
        &self,
        segment: u16,
        bus: u8,
        device: u8,
        function: u8,
        offset: u16,
        value: u16,
    ) {
        let address = PciAddress::new(segment, bus, device, function);
        debug!("WritePCI16 {} {} {}", address, offset, value);
        unsafe {
            let mut read = distros_pci_access::access().read(address, offset);
            read.set_bits(0..16, value as u32);
            distros_pci_access::access().write(address, offset, read);
        }
    }

    fn write_pci_u32(
        &self,
        segment: u16,
        bus: u8,
        device: u8,
        function: u8,
        offset: u16,
        value: u32,
    ) {
        let address = PciAddress::new(segment, bus, device, function);
        debug!("WritePCI32 {} {} {}", address, offset, value);
        unsafe {
            distros_pci_access::access().write(address, offset, value);
        }
    }
}
