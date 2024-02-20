use x86_64::instructions::interrupts::without_interrupts;
use x86_64::instructions::port::Port;

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
#[repr(u8)]
pub enum Channel {
    Channel0 = 0b00,
    Channel2 = 0b10,
}

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
#[repr(u8)]
pub enum OperatingMode {
    InterruptOnTerminalCount = 0b000,
    HardwareReTriggerableOneShot = 0b001,
    RateGenerator = 0b010,
    SquareWaveGenerator = 0b011,
    SoftwareTriggeredStrobe = 0b100,
    HardwareTriggeredStrobe = 0b101,
}

pub struct Pit {
    cmd: Port<u8>,
    channel_0: Port<u8>,
    channel_2: Port<u8>,
    speaker: Port<u8>,
}

impl Pit {
    pub const FREQUENCY: usize = 1193180;

    pub const fn new() -> Pit {
        Pit {
            channel_0: Port::new(0x40),
            channel_2: Port::new(0x42),
            cmd: Port::new(0x43),
            speaker: Port::new(0x61),
        }
    }

    pub fn set(&mut self, channel: Channel, value: u16) {
        without_interrupts(|| match channel {
            Channel::Channel0 => unsafe {
                self.channel_0.write((value & 0xFF) as u8);
                self.channel_0.write(((value & 0xFF00) >> 8) as u8);
            },
            Channel::Channel2 => unsafe {
                self.channel_2.write((value & 0xFF) as u8);
                self.channel_2.write(((value & 0xFF00) >> 8) as u8);
            },
        });
    }

    pub fn get(&mut self, channel: Channel) -> u16 {
        without_interrupts(|| match channel {
            Channel::Channel0 => unsafe {
                self.cmd.write(0b00000000u8);
                self.channel_0.read() as u16 | ((self.channel_0.read() as u16) << 8)
            },
            Channel::Channel2 => unsafe {
                self.cmd.write(0b10000000u8);
                self.channel_2.read() as u16 | ((self.channel_2.read() as u16) << 8)
            },
        })
    }

    pub fn setup(
        &mut self,
        channel: Channel,
        operating_mode: OperatingMode,
        is_bcd: bool,
        comp: u16,
    ) {
        let cmd = is_bcd as u8 | ((operating_mode as u8) << 1) | 0b11 << 4 | ((channel as u8) << 6);
        without_interrupts(|| unsafe {
            self.cmd.write(cmd);
            match channel {
                Channel::Channel0 => {
                    self.channel_0.write(comp as u8);
                    self.channel_0.write((comp >> 8) as u8);
                }
                Channel::Channel2 => {
                    self.channel_2.write(comp as u8);
                    self.channel_2.write((comp >> 8) as u8);
                }
            }
        });
    }

    pub fn enable_speaker(&mut self, enable: bool) {
        without_interrupts(|| unsafe {
            if enable {
                let tmp = self.speaker.read();
                if tmp != (tmp | 3) {
                    self.speaker.write(tmp | 3);
                }
            } else {
                let value = self.speaker.read();
                self.speaker.write(value & 0xFC);
            }
        });
    }
}
