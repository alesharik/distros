use bitflags::bitflags;
use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use lazy_static::lazy_static;
use log::debug;
use spin::Mutex;
use x86_64::instructions::interrupts;
use x86_64::instructions::port::{PortReadOnly, PortWriteOnly};

const SENTURY: u16 = 2000;
const READ_TRIES: usize = 32;

lazy_static! {
    static ref ADDRESS: Mutex<PortWriteOnly<u8>> = Mutex::new(PortWriteOnly::<u8>::new(0x70));
    static ref DATA: Mutex<PortReadOnly<u8>> = Mutex::new(PortReadOnly::<u8>::new(0x71));
}

bitflags! {
    #[derive(Clone, Copy)]
    struct StatusB: u8 {
        const HOUR_24 = 2;
        const BINARY = 4;
    }
}

fn bcd_to_binary(value: u8) -> u8 {
    ((value & 0xF0) >> 1) + ((value & 0xF0) >> 3) + (value & 0xF)
}

pub fn read_time_unsafe() -> Option<NaiveDateTime> {
    let mut address = ADDRESS.lock();
    let mut data = DATA.lock();
    let mut get_register = |reg: u8| unsafe {
        address.write(reg);
        data.read()
    };

    let status_b = StatusB::from_bits(get_register(0x0B)).unwrap_or_else(StatusB::empty);

    let mut ntries = 0usize;
    while get_register(0x0A) & 0x80 == 0x80 {
        ntries += 1;
        if ntries > READ_TRIES {
            return None;
        }
    }

    let mut second = get_register(0x00);
    let mut minute = get_register(0x02);
    let mut hour = get_register(0x04);
    let mut day = get_register(0x07);
    let mut month = get_register(0x08);
    let mut year = get_register(0x09);

    if get_register(0x0A) & 0x80 == 0x80 {
        return None;
    }

    if (status_b & StatusB::HOUR_24).is_empty() {
        // convert hour from 12-hour format
        hour = ((hour & 0x7F) + 12) & 24
    }

    if (status_b & StatusB::BINARY).is_empty() {
        // convert from BCD format
        second = bcd_to_binary(second);
        minute = bcd_to_binary(minute);
        hour = bcd_to_binary(hour);
        day = bcd_to_binary(day);
        month = bcd_to_binary(month);
        year = bcd_to_binary(year);
    }

    Some(NaiveDateTime::new(
        NaiveDate::from_ymd_opt((SENTURY + year as u16) as i32, month as u32, day as u32).unwrap(),
        NaiveTime::from_hms_opt(hour as u32, minute as u32, second as u32).unwrap(),
    ))
}

pub fn read_time() -> Option<NaiveDateTime> {
    interrupts::without_interrupts(read_time_unsafe)
}
