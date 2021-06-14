use core::fmt::Write;
use lazy_static::lazy_static;
use spin::Mutex;
use vga::colors::{Color16, TextModeColor};
use vga::writers::{ScreenCharacter, Text80x25, TextWriter};

lazy_static! {
    static ref VGA: Mutex<Vga> = Mutex::new(Vga::new());
}

struct Vga {
    text: Text80x25,
    col: usize,
    row: usize,
    color: TextModeColor,
}

const WIDTH: usize = 80;
const HEIGHT: usize = 25;
const DEFAULT_COLOR: Color16 = Color16::Yellow;

impl Vga {
    fn new() -> Self {
        let text = Text80x25::new();
        text.enable_cursor();
        Vga {
            text,
            row: 0,
            col: 0,
            color: TextModeColor::new(DEFAULT_COLOR, Color16::Black),
        }
    }

    fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => {
                self.new_line();
                self.text.set_cursor_position(self.row, self.col);
            }
            byte => {
                if self.row >= WIDTH {
                    self.new_line()
                }
                self.text.write_character(
                    self.row,
                    self.col,
                    ScreenCharacter::new(byte, self.color),
                );
                self.text.set_cursor_position(self.row, self.col);
                self.row += 1;
            }
        }
    }

    fn new_line(&mut self) {
        self.row = 0;
        if self.col >= HEIGHT - 1 {
            for h in 1..HEIGHT {
                for w in 0..WIDTH {
                    self.text
                        .write_character(w, h - 1, self.text.read_character(w, h))
                }
            }
            for w in 0..WIDTH {
                self.text
                    .write_character(w, HEIGHT - 1, ScreenCharacter::new(b' ', self.color))
            }
        } else {
            self.col += 1;
        }
    }

    #[allow(dead_code)]
    fn set_background(&mut self, color: Color16) {
        self.color.set_background(color)
    }

    fn set_foreground(&mut self, color: Color16) {
        self.color.set_foreground(color)
    }
}

impl Write for Vga {
    fn write_str(&mut self, text: &str) -> core::fmt::Result {
        for s in text.bytes() {
            match s {
                0x20..=0x7e | b'\n' => self.write_byte(s),
                _ => self.write_byte(0xfe),
            }
        }
        Ok(())
    }
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::vga::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn _print(args: core::fmt::Arguments) {
    VGA.lock().write_fmt(args).unwrap();
}

#[macro_export]
macro_rules! eprint {
    ($($arg:tt)*) => ($crate::vga::_eprint(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! eprintln {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::eprint!("{}\n", format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn _eprint(args: core::fmt::Arguments) {
    let mut guard = VGA.lock();
    guard.set_foreground(Color16::Red);
    guard.write_fmt(args).unwrap();
    guard.set_foreground(DEFAULT_COLOR);
}

#[macro_export]
macro_rules! kblog {
    ($mod:tt) => ($crate::vga::_kblog($mod, format_args!("")));
    ($mod:tt, $($arg:tt)*) => ($crate::vga::_kblog($mod, format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn _kblog(module: &str, args: core::fmt::Arguments) {
    let mut guard = VGA.lock();
    guard.set_foreground(Color16::Green);
    guard.write_char('[').unwrap();
    guard.write_str(module).unwrap();
    guard.write_str("] ").unwrap();
    guard.set_foreground(DEFAULT_COLOR);
    guard.write_fmt(args).unwrap();
    guard.write_char('\n').unwrap();
}
