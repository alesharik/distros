use chrono::{NaiveDateTime, NaiveDate, NaiveTime};
use spin::Mutex;
use x86_64::instructions::port::{PortReadOnly, PortWriteOnly};

const SENTURY: u16 = 2000;

lazy_static!(
    static ref ADDRESS: Mutex<PortWriteOnly<u8>> = Mutex::new(PortWriteOnly::<u8>::new(0x70));
    static ref DATA: Mutex<PortReadOnly<u8>> = Mutex::new(PortReadOnly::<u8>::new(0x71));
);

bitflags! {
    struct StatusB: u8 {
        const HOUR_24 = 2;
        const BINARY = 4;
    }
}

fn bcd_to_binary(value: u8) -> u8 {
    ((value & 0xF0) >> 1 ) + ((value & 0xF0) >> 3) + (value & 0xF)
}

pub fn read_time() -> NaiveDateTime {
    let mut address = ADDRESS.lock();
    let mut data = DATA.lock();
    let mut get_register = |reg: u8| {
        unsafe {
            address.write(reg);
            data.read()
        }
    };

    let status_b = StatusB::from_bits(get_register(0x0B)).unwrap_or(StatusB::empty());

    while get_register(0x0A) & 0x80 == 0x80 {};

    let mut second = get_register(0x00);
    let mut minute = get_register(0x02);
    let mut hour = get_register(0x04);
    let mut day = get_register(0x07);
    let mut month = get_register(0x08);
    let mut year = get_register(0x09);
    loop {
        while get_register(0x0A) & 0x80 == 0x80 {};

        let cur_second = get_register(0x00);
        let cur_minute = get_register(0x02);
        let cur_hour = get_register(0x04);
        let cur_day = get_register(0x07);
        let cur_month = get_register(0x08);
        let cur_year = get_register(0x09);

        // values from two reads identical? Then we received them between RTC cycles, otherwise they are not valid
        if second == cur_second && minute == cur_minute && hour == cur_hour && day == cur_day && month == cur_month && year == cur_year {
            break
        } else {
            second = cur_second;
            minute = cur_minute;
            hour = cur_hour;
            day = cur_day;
            month = cur_month;
            year = cur_year;
        }
    }

    if (status_b & StatusB::BINARY).is_empty() { // convert from BCD format
        second = bcd_to_binary(second);
        minute = bcd_to_binary(minute);
        hour = bcd_to_binary(hour);
        day = bcd_to_binary(day);
        month = bcd_to_binary(month);
        year = bcd_to_binary(year);
    }

    if (status_b & StatusB::HOUR_24).is_empty() { // convert hour from 12-hour format
        hour = ((hour & 0x7F) + 12) & 24
    }

    NaiveDateTime::new(
        NaiveDate::from_ymd((SENTURY + year as u16) as i32, month as u32, day as u32),
        NaiveTime::from_hms(hour as u32, minute as u32, second as u32)
    )
}