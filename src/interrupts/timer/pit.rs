use spin::Mutex;
use x86_64::instructions::port::PortWriteOnly;
use crate::interrupts::Irq;

const PIT_FREQ: u32 = 1193182;
const COUNTER0_FREQ: u32 = 1000;

const PIT_CMD_BINARY: u8 = 0x00;
const PIT_CMD_BCD: u8 = 0x01;

const PIT_CMD_MODE_INTERRUPT: u8 = 0x00;
const PIT_CMD_MODE_ONESHOT: u8 = 0x02;
const PIT_CMD_MODE_RATE: u8 = 0x04;
const PIT_CMD_MODE_SQUARE: u8 = 0x06;
const PIT_CMD_MODE_SOFT_STROBE: u8 = 0x08;
const PIT_CMD_MODE_HARD_STROBE: u8 = 0x0a;

const PIT_CMD_LATCH: u8 = 0x00;
const PIT_CMD_RW_LOW: u8 = 0x10;
const PIT_CMD_RW_HI: u8 = 0x20;
const PIT_CMD_RW_BOTH: u8 = 0x30;

const PIT_CMD_COUNTER0: u8 = 0x00;
const PIT_CMD_COUNTER1: u8 = 0x40;
const PIT_CMD_COUNTER2: u8 = 0x80;
const PIT_CMD_READBACK: u8 = 0xc0;

const PIT_IRQ: u8 = 0;

lazy_static!(
    static ref COUNTER0: Mutex<PortWriteOnly<u8>> = Mutex::new(PortWriteOnly::<u8>::new(0x40));
    static ref COUNTER_CTRL: Mutex<PortWriteOnly<u8>> = Mutex::new(PortWriteOnly::<u8>::new(0x43));
);

pub fn init_pit() -> Irq {
    let mut counter0 = COUNTER0.lock();
    let mut counter_ctrl = COUNTER_CTRL.lock();
    let divisor = PIT_FREQ / COUNTER0_FREQ;
    unsafe {
        counter_ctrl.write(PIT_CMD_BINARY | PIT_CMD_MODE_SQUARE | PIT_CMD_RW_BOTH | PIT_CMD_COUNTER0);
        counter0.write(divisor as u8);
        counter0.write((divisor >> 8) as u8)
    }
    kblog!("PIT", "PIT started");
    return Irq(PIT_IRQ);
}