use crate::capabilities::CapabilitiesRegister;
use crate::timer::Timer;
use bit_field::BitField;
use log::debug;
use x86_64::VirtAddr;

pub struct Hpet {
    address: VirtAddr,
    caps: CapabilitiesRegister,
    enabled: bool,
    legacy_replacement: bool,
}

const CONFIG_REG_OFFSET: u64 = 0x010u64;
const GIS_REG_OFFSET: u64 = 0x020u64;
const MAIN_COUNTER_REG_OFFSET: u64 = 0x0F0u64;

impl Hpet {
    pub fn new(address: VirtAddr) -> Self {
        let caps = CapabilitiesRegister::read(address);
        let cfg_reg = address + CONFIG_REG_OFFSET;
        let cfg: *const u64 = cfg_reg.as_ptr();
        unsafe {
            Hpet {
                caps,
                address,
                enabled: (&*cfg).get_bit(0),
                legacy_replacement: (&*cfg).get_bit(1),
            }
        }
    }

    /// Main counter tick period in femtoseconds (10^-15 seconds).
    pub fn period(&self) -> u32 {
        self.caps.period()
    }

    pub fn vendor(&self) -> u16 {
        self.caps.vendor()
    }

    pub fn capable_legacy_replacement(&self) -> bool {
        self.caps.capable_legacy_replacement()
    }

    pub fn capable_64bit(&self) -> bool {
        self.caps.capable_64bit()
    }

    pub fn timers(&self) -> u8 {
        self.caps.timers() + 1
    }

    pub fn revision(&self) -> u8 {
        self.caps.revision()
    }

    pub fn enable_legacy_replacement(&mut self) {
        if !self.capable_legacy_replacement() {
            return;
        }
        let addr = self.address + CONFIG_REG_OFFSET;
        let mut ptr: *mut u64 = addr.as_mut_ptr();
        unsafe {
            (&mut *ptr).set_bit(1, true);
        }
        self.legacy_replacement = true;
    }

    pub fn disable_legacy_replacement(&mut self) {
        if !self.capable_legacy_replacement() {
            return;
        }
        let addr = self.address + CONFIG_REG_OFFSET;
        let mut ptr: *mut u64 = addr.as_mut_ptr();
        unsafe {
            (&mut *ptr).set_bit(1, false);
        }
        self.legacy_replacement = false;
    }

    pub fn legacy_replacement_enabled(&self) -> bool {
        self.legacy_replacement
    }

    pub fn enable(&mut self) {
        let addr = self.address + CONFIG_REG_OFFSET;
        let mut ptr: *mut u64 = addr.as_mut_ptr();
        unsafe {
            (&mut *ptr).set_bit(0, true);
        }
        self.enabled = true;
    }

    pub fn disable(&mut self) {
        let addr = self.address + CONFIG_REG_OFFSET;
        let mut ptr: *mut u64 = addr.as_mut_ptr();
        unsafe {
            (&mut *ptr).set_bit(0, false);
        }
        self.enabled = false;
    }

    pub fn enabled(&self) -> bool {
        self.enabled
    }

    pub fn level_int_active(&self, timer: u8) -> bool {
        if timer >= 32 {
            panic!("GIS register max timer value is 32, got {}", timer);
        }
        let addr = self.address + GIS_REG_OFFSET;
        let mut ptr: *mut u64 = addr.as_mut_ptr();
        unsafe { (&mut *ptr).get_bit(timer as usize) }
    }

    pub fn clear_level_int(&self, timer: u8) {
        if timer >= 32 {
            panic!("GIS register max timer value is 32, got {}", timer);
        }
        let addr = self.address + GIS_REG_OFFSET;
        let mut ptr: *mut u64 = addr.as_mut_ptr();
        unsafe {
            (&mut *ptr).set_bit(timer as usize, true);
        }
    }

    pub fn get_main_counter(&self) -> u64 {
        let reg = self.address + MAIN_COUNTER_REG_OFFSET;
        return if self.capable_64bit() {
            let ptr: *const u32 = reg.as_ptr();
            unsafe { *ptr as u64 }
        } else {
            let ptr: *const u64 = reg.as_ptr();
            unsafe { *ptr }
        };
    }

    pub fn set_main_counter(&mut self, value: u64) {
        let enabled = self.enabled;
        if enabled {
            self.disable();
        }
        let reg = self.address + MAIN_COUNTER_REG_OFFSET;
        if self.capable_64bit() {
            let ptr: *mut u32 = reg.as_mut_ptr();
            unsafe { *ptr = value as u32 }
        } else {
            let ptr: *mut u64 = reg.as_mut_ptr();
            unsafe { *ptr = value }
        }
        if enabled {
            self.enable();
        }
    }

    pub fn get_timer(&self, timer: u8) -> Option<Timer> {
        if timer >= self.timers() {
            return None;
        }
        let reg = self.address + (0x100u64 + 0x20u64 * timer as u64);
        let ptr: *const u64 = reg.as_ptr();
        unsafe { Some(Timer::new(timer, *ptr)) }
    }

    pub fn set_timer(&mut self, timer: Timer) {
        let reg = self.address + (0x100u64 + 0x20u64 * timer.timer() as u64);
        let ptr: *mut u64 = reg.as_mut_ptr();
        unsafe { *ptr = timer.cfg() }
    }

    pub fn get_timer_comparator(&mut self, timer: Timer) -> u64 {
        let reg = self.address + (0x108u64 + 0x20u64 * timer.timer() as u64);
        if timer.is_64bit() && !timer.is_32_bit_mode_enabled() {
            let ptr: *const u64 = reg.as_ptr();
            unsafe { *ptr }
        } else {
            let ptr: *const u32 = reg.as_ptr();
            unsafe { *ptr as u64 }
        }
    }

    pub fn set_timer_comparator(&mut self, timer: Timer, value: u64) {
        let reg = self.address + (0x108u64 + 0x20u64 * timer.timer() as u64);
        if timer.is_64bit() && !timer.is_32_bit_mode_enabled() {
            let ptr: *mut u64 = reg.as_mut_ptr();
            unsafe {
                *ptr = value;
            }
        } else {
            let ptr: *mut u32 = reg.as_mut_ptr();
            unsafe { *ptr = value as u32 }
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = Timer> + '_ {
        TimerIter {
            hpet: self,
            current: 0,
        }
    }
}

struct TimerIter<'a> {
    hpet: &'a Hpet,
    current: u8,
}

impl<'a> Iterator for TimerIter<'a> {
    type Item = Timer;

    fn next(&mut self) -> Option<Self::Item> {
        match self.hpet.get_timer(self.current) {
            None => None,
            Some(tim) => {
                self.current += 1;
                Some(tim)
            }
        }
    }
}
