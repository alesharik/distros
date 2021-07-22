use x86_64::{PhysAddr, VirtAddr};
use smbioslib::MemoryMapper;
use crate::memory;

struct MemoryMapperImpl {}

impl MemoryMapper for MemoryMapperImpl {
    #[inline]
    fn map_block(&mut self, addr: PhysAddr, _size: usize) -> VirtAddr {
        memory::map_physical_address(addr)
    }
}

pub fn init() {
    let mut mapper = MemoryMapperImpl {};
    let t1 = smbioslib::table_load_from_device(&mut mapper).unwrap();
    let test = format!("{:#?}", t1);
    info!("test");
}