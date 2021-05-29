use acpi::HpetInfo;
use crate::memory;
use x86_64::{PhysAddr, VirtAddr};
use bit_field::BitField;
use core::ops::Not;
use crate::interrupts::Irq;

const RTC_COMP: u8 = 0;

struct TimerConfiguration(u64);

impl TimerConfiguration {
    fn supports_apic_line(&self, line: Irq) -> bool {
        self.0.get_bit((32 + line.0) as usize)
    }

    fn supports_fsb(&self) -> bool {
        self.0.get_bit(15)
    }

    fn set_fsb_interrupts(&mut self, val: bool) {
        self.0.set_bit(14, val);
    }

    fn set_apic_line(&mut self, line: u8) {
        self.0.set_bits(9..13, line as u64);
    }

    fn set_32_mode(&mut self, val: bool) {
        self.0.set_bit(8, val);
    }

    fn allow_set_accumulator(&mut self) {
        self.0.set_bit(6, true);
    }

    fn is_64bit(&self) -> bool {
        self.0.get_bit(5)
    }

    fn supports_periodic(&self) -> bool {
        self.0.get_bit(4)
    }

    fn set_periodic(&mut self, val: bool) {
        self.0.set_bit(3, val);
    }

    fn set_interrupts_enabled(&mut self, val: bool) {
        self.0.set_bit(2, val);
    }

    fn set_level_trigger(&mut self, val: bool) {
        self.0.set_bit(1, val);
    }
}

fn find_hpet_periodic_timer(info: &HpetInfo, addr: &VirtAddr, off: u8) -> u8 {
    unsafe {
        let mut cur_off = 0;
        for cmp in 0..info.num_comparators() {
            let cmp_cap: VirtAddr = addr.clone() + 0x100 as usize + (0x20 * cmp) as usize;
            let cfg = TimerConfiguration(*cmp_cap.as_ptr());
            if cfg.supports_periodic() {
                cur_off += 1;
                if cur_off > off {
                    return cmp;
                }
            }
        }
    }
    panic!("HPET does not have comparator for RTC")
}

pub fn init_hpet_rtc(info: &HpetInfo) -> Irq {
    let addr: VirtAddr = memory::map_physical_address(PhysAddr::new(info.base_address as u64));
    let comparator = find_hpet_periodic_timer(info, &addr, RTC_COMP);
    let period = unsafe { *(addr.as_ptr::<u32>()) };
    let frequency = (10 as u64).pow(15) / period as u64;
    let target = frequency / 1000; // 1kHz divider
    if target < info.clock_tick_unit as u64 {
        panic!("RTC divider {} < min tick count {}", target, info.clock_tick_unit)
    }
    let cmp_cap: VirtAddr = addr + 0x100 as usize + (0x20 * comparator) as usize;
    let mut cfg = TimerConfiguration(unsafe { *cmp_cap.as_ptr() });
    cfg.set_periodic(true);
    cfg.set_interrupts_enabled(true);
    cfg.allow_set_accumulator();

    let cmp_val: VirtAddr = addr + 0x100 as usize + (0x20 * comparator) as usize;

    for irq in 0..24 {
        let irq = Irq::from_raw(irq);
        if cfg.supports_apic_line(irq) && !irq.has_handler() {
            cfg.set_apic_line(irq.0 as u8);
            unsafe { *cmp_cap.as_mut_ptr() = cfg.0 }
            unsafe {
                *cmp_val.as_mut_ptr::<u64>() = target;
                *cmp_val.as_mut_ptr::<u64>() = target;
            }
            return irq
        }
    }
    panic!("Cannot map HPET RTC timer to irq")
}

pub fn start_hpet(info: &HpetInfo) {
    let addr: VirtAddr = memory::map_physical_address(PhysAddr::new(info.base_address as u64)) + 0x010 as usize;
    let counter_addr: VirtAddr = memory::map_physical_address(PhysAddr::new(info.base_address as u64)) + 0x0F0 as usize;
    unsafe {
        let mut val = *addr.as_ptr::<u64>();
        val |= 1;
        val &= (2 as u64).not();
        *(addr.as_mut_ptr::<u64>()) = val;
        *(counter_addr.as_mut_ptr::<u64>()) = 0;
    }
    kblog!("HPET", "HPET started");
}