use core::fmt::Write;
use lazy_static::lazy_static;
use spin::Mutex;
use crate::driver::gpu::VesaFrameBuffer;
use crate::gui::TextDisplay;

lazy_static! {
    static ref GPU: Mutex<Option<TextDisplay<VesaFrameBuffer>>> = Mutex::new(None);
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::logging::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn _print(args: core::fmt::Arguments) {
    let mut display = GPU.lock();
    if let Some(display) = display.as_mut() {
        display.write_fmt(args).unwrap();
    }
}

#[macro_export]
macro_rules! eprint {
    ($($arg:tt)*) => ($crate::logging::_eprint(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! eprintln {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::eprint!("{}\n", format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn _eprint(args: core::fmt::Arguments) {
    let mut display = GPU.lock();
    if let Some(display) = display.as_mut() {
        display.write_fmt(args).unwrap();
    }
}

#[macro_export]
macro_rules! kblog {
    ($mod:tt) => ($crate::logging::_kblog($mod, format_args!("")));
    ($mod:tt, $($arg:tt)*) => ($crate::logging::_kblog($mod, format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn _kblog(module: &str, args: core::fmt::Arguments) {
    let mut display = GPU.lock();
    if let Some(display) = display.as_mut() {
        display.write_char('[').unwrap();
        display.write_str(module).unwrap();
        display.write_str("] ").unwrap();
        display.write_fmt(args).unwrap();
        display.write_char('\n').unwrap();
    }
}

pub fn init(fb: VesaFrameBuffer) {
    let mut display = GPU.lock();
    *display = Some(TextDisplay::new(fb));
}