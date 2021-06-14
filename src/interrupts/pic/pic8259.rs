use crate::kblog;
use spin::Mutex;
use x86_64::instructions::port::Port;

const PIC1: u16 = 0x20;
const PIC2: u16 = 0xA0;
lazy_static! {
    static ref PIC1_COMMAND: Mutex<Port<u8>> = Mutex::new(Port::new(PIC1));
    static ref PIC2_COMMAND: Mutex<Port<u8>> = Mutex::new(Port::new(PIC2));
    static ref PIC1_DATA: Mutex<Port<u8>> = Mutex::new(Port::new(PIC1 + 1));
    static ref PIC2_DATA: Mutex<Port<u8>> = Mutex::new(Port::new(PIC2 + 1));
}

fn set_mask(line: u8) {
    let mut port = if line < 8 {
        PIC1_DATA.lock()
    } else {
        PIC2_DATA.lock()
    };
    let line = if line >= 8 { line - 8 } else { line };
    unsafe {
        let val = port.read() | (1 << line);
        port.write(val)
    }
}

fn disable_pic() {
    let mut pic1 = PIC1_DATA.lock();
    let mut pic2 = PIC2_DATA.lock();
    unsafe {
        pic1.write(0xff);
        pic2.write(0xff);
    }
}

pub fn disable() {
    for irq in 0..16 {
        set_mask(irq);
    }
    kblog!("PIC8259", "IRQs masked");
    disable_pic();
    kblog!("PIC8259", "PIC disabled");
}
