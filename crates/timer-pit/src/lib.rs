#![no_std]

use crate::pit::{Channel, OperatingMode, Pit};
use spin::Mutex;

mod pit;

static PIT: Mutex<Pit> = Mutex::new(Pit::new());

pub fn set(channel: Channel, value: u16) {
    let mut pit = PIT.lock();
    pit.set(channel, value);
}

pub fn get(channel: Channel) -> u16 {
    let mut pit = PIT.lock();
    pit.get(channel)
}

pub fn setup_timer(operating_mode: OperatingMode, is_bcd: bool, comp: u16) {
    let mut pit = PIT.lock();
    pit.setup(Channel::Channel0, operating_mode, is_bcd, comp);
}

pub fn setup_sound(frequency: u16) {
    let mut pit = PIT.lock();
    pit.setup(
        Channel::Channel2,
        OperatingMode::SquareWaveGenerator,
        false,
        (Pit::FREQUENCY / frequency as usize) as u16,
    );
}

pub fn enable_speaker(enable: bool) {
    let mut pit = PIT.lock();
    pit.enable_speaker(enable);
}
