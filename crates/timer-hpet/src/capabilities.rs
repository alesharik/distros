use bit_field::BitField;
use x86_64::VirtAddr;

pub struct CapabilitiesRegister(u64);

impl CapabilitiesRegister {
    pub const fn new(reg: u64) -> Self {
        CapabilitiesRegister(reg)
    }

    pub fn read(base: VirtAddr) -> Self {
        let ptr: *const u64 = base.as_ptr();
        unsafe { CapabilitiesRegister(*ptr) }
    }

    pub fn period(&self) -> u32 {
        self.0.get_bits(32..=63) as u32
    }

    pub fn vendor(&self) -> u16 {
        self.0.get_bits(16..=31) as u16
    }

    pub fn capable_legacy_replacement(&self) -> bool {
        self.0.get_bit(15)
    }

    pub fn capable_64bit(&self) -> bool {
        self.0.get_bit(13)
    }

    pub fn timers(&self) -> u8 {
        self.0.get_bits(8..=12) as u8
    }

    pub fn revision(&self) -> u8 {
        self.0.get_bits(0..=7) as u8
    }
}
