use x86_64::{PhysAddr, VirtAddr};
use smbioslib::*;
use crate::memory;
use crate::flow::{Message, ContentProvider};
use core::fmt::{Debug, Formatter};
use core::marker::PhantomData;
use core::ops::Deref;

struct MemoryMapperImpl {}

impl MemoryMapper for MemoryMapperImpl {
    #[inline]
    fn map_block(&mut self, addr: PhysAddr, _size: usize) -> VirtAddr {
        memory::map_physical_address(addr)
    }
}

pub struct SMBiosMessage<T> where for<'a> T: SMBiosStruct<'a> + Send + Sync {
    undef_struct: UndefinedStruct,
    typ: PhantomData<T>
}

// impl<T> Deref for SMBiosMessage<T> where for<'a> T: SMBiosStruct<'a> + Send + Sync {
//     type Target = T;
//
//     fn deref(&self) -> &Self::Target {
//         &Self::Target::new(&self.undef_struct)
//     }
// }

impl<T> Debug for SMBiosMessage<T> where for<'a> T: SMBiosStruct<'a> + Debug + Send + Sync {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:?}", self.deref())
    }
}

impl<T> Message for SMBiosMessage<T> where for<'a> T: SMBiosStruct<'a> + Debug + Send + Sync {}

macro_rules! wrap {
    ($struct:ident, $var:ident) => {
        {
            ContentProvider::new($struct::new(&$var.parts().clone()))
        }
    };
}

fn define_item(item: DefinedStruct<'_>) {
    if let DefinedStruct::SystemReset(reset) = item {
        // let msg = SMBiosMessage::<SMBiosSystemReset<'_>> {
        //     undef_struct: reset.parts().clone(),
        //     typ: PhantomData::<SMBiosSystemReset>::default(),
        // };
        // let provider = ContentProvider::new(msg);
    }
}

pub fn init() {
    info!("Setting up");
    let mut mapper = MemoryMapperImpl {};
    let data = match smbioslib::table_load_from_device(&mut mapper) {
        Ok(data) => data,
        Err(e) => {
            info!("Failed with error {:?}", e);
            return;
        }
    };
    let table: DefinedStructTable<'_> = data.iter().collect();
    for item in table.into_iter() {
        define_item(item)
    }
    info!("Setup complete");
}