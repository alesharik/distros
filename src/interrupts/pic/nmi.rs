use core::ops::Not;
use spin::Mutex;
use x86_64::instructions::port::{Port, PortReadOnly};

bitflags! {
    pub struct StatusA: u8 {
        const ALTERNATE_HOST_RESET = 0b00000001;
        const ALTERNATE_GATE_A20 = 0b00000010;
        const SECURITY_LOCK = 0b00001000;
        const WATCHDOG = 0b00010000;
        const HDD_1_DRIVE_ACTIVITY = 0b01000000;
        const HDD_2_DRIVE_ACTIVITY = 0b10000000;
    }
}

bitflags! {
    pub struct StatusB: u8 {
        const TIMER_2_TIED_TO_SPEAKER = 0b00000001;
        const SPEAKER_DATA_ENABLE = 0b00000010;
        const PARITY_CHECK_ENABLE = 0b00000100;
        const CHANNEL_CHECK_ENABLE = 0b00001000;
        const REFRESH_REQUEST = 0b00010000;
        const TIMER_2_OUTPUT = 0b00100000;
        const CHANNEL_CHECK = 0b01000000;
        const PARITY_CHECK = 0b10000000;
    }
}

lazy_static! {
    static ref CONTROL_PORT: Mutex<Port<u8>> = Mutex::new(Port::<u8>::new(0x70));
    static ref STATUS_A: Mutex<PortReadOnly<u8>> = Mutex::new(PortReadOnly::<u8>::new(0x92));
    static ref STATUS_B: Mutex<PortReadOnly<u8>> = Mutex::new(PortReadOnly::<u8>::new(0x61));
    static ref ENABLED: Mutex<bool> = Mutex::new(false);
}

pub fn nmi_enable() {
    let mut enabled = ENABLED.lock();
    if enabled.clone() {
        return;
    }
    let mut ctl = CONTROL_PORT.lock();
    unsafe {
        let val = ctl.read();
        ctl.write(val & 0x7F);
    }
    *enabled = true;
}

pub fn nmi_disable() {
    let mut enabled = ENABLED.lock();
    if enabled.not() {
        return;
    }
    let mut ctl = CONTROL_PORT.lock();
    unsafe {
        let val = ctl.read();
        ctl.write(val | 0x80);
    }
    *enabled = true;
}

#[must_use]
pub fn nmi_enabled() -> bool {
    ENABLED.lock().clone()
}

#[must_use]
pub fn nmi_status() -> (StatusA, StatusB) {
    unsafe {
        let mut status_a = STATUS_A.lock();
        let mut status_b = STATUS_B.lock();
        (
            StatusA::from_bits(status_a.read()).unwrap_or(StatusA::empty()),
            StatusB::from_bits(status_b.read()).unwrap_or(StatusB::empty()),
        )
    }
}
