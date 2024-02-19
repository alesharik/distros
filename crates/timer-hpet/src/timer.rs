use bit_field::BitField;
use core::fmt::{Debug, Formatter};

#[derive(Copy, Clone)]
pub struct Timer {
    cfg: u64,
    timer: u8,
}

impl Timer {
    pub(crate) const fn new(timer: u8, cfg: u64) -> Self {
        Timer { cfg, timer }
    }

    pub(crate) fn cfg(&self) -> u64 {
        self.cfg
    }

    pub(crate) fn timer(&self) -> u8 {
        self.timer
    }

    #[inline]
    pub fn is_64bit(&self) -> bool {
        self.cfg.get_bit(5)
    }

    #[inline]
    pub fn supports_apic_line(&self, line: u8) -> bool {
        self.cfg.get_bit((32 + line) as usize)
    }

    #[inline]
    pub fn first_available_apic_line(&self) -> Option<u8> {
        for i in 0..32 {
            if self.supports_apic_line(i) {
                return Some(i);
            }
        }
        None
    }

    #[inline]
    pub fn supports_periodic(&self) -> bool {
        self.cfg.get_bit(4)
    }

    #[inline]
    pub fn supports_fsb(&self) -> bool {
        self.cfg.get_bit(15)
    }

    pub fn set_fsb_interrupts(mut self, val: bool) -> Self {
        self.cfg.set_bit(14, val);
        self
    }

    #[inline]
    pub fn fsb_interrupts_enabled(&self) -> bool {
        self.cfg.get_bit(14)
    }

    pub fn set_apic_line(mut self, line: u8) -> Self {
        self.cfg.set_bits(9..14, line as u64);
        self
    }

    #[inline]
    pub fn apic_line(&self) -> u8 {
        self.cfg.get_bits(9..14) as u8
    }

    pub fn set_32_mode(mut self, val: bool) -> Self {
        self.cfg.set_bit(8, val);
        self
    }

    #[inline]
    pub fn is_32_bit_mode_enabled(&self) -> bool {
        self.cfg.get_bit(8)
    }

    pub fn allow_set_accumulator(mut self) -> Self {
        self.cfg.set_bit(6, true);
        self
    }

    pub fn set_periodic(mut self, val: bool) -> Self {
        self.cfg.set_bit(3, val);
        self
    }

    #[inline]
    pub fn is_periodic(&self) -> bool {
        self.cfg.get_bit(3)
    }

    pub fn set_interrupts_enabled(mut self, val: bool) -> Self {
        self.cfg.set_bit(2, val);
        self
    }

    pub fn interrupts_enabled(&self) -> bool {
        self.cfg.get_bit(2)
    }

    pub fn set_level_trigger(mut self, val: bool) -> Self {
        self.cfg.set_bit(1, val);
        self
    }

    pub fn level_triggered(&self) -> bool {
        self.cfg.get_bit(1)
    }
}

impl Debug for Timer {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Timer")
            .field("timer", &self.timer())
            .field("is_64bit", &self.is_64bit())
            .field("supports_periodic", &self.supports_periodic())
            .field("supports_fsb", &self.supports_fsb())
            .field("fsb_interrupts_enabled", &self.fsb_interrupts_enabled())
            .field("apic_line", &self.apic_line())
            .field("is_32_bit_mode_enabled", &self.is_32_bit_mode_enabled())
            .field("is_periodic", &self.is_periodic())
            .field("interrupts_enabled", &self.interrupts_enabled())
            .field("level_triggered", &self.level_triggered())
            .finish()
    }
}
