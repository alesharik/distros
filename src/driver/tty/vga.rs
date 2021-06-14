use crate::driver::tty::TtyScreen;
use vga::colors::{Color16, TextModeColor};
use vga::writers::{ScreenCharacter, Text80x25, TextWriter};
use vte::ansi::{CursorShape, CursorStyle, Rgb};

const DEFAULT_FOREGROUND: Color16 = Color16::White;
const DEFAULT_BACKGROUND: Color16 = Color16::Black;

pub struct VgaTextScreen {
    col: usize,
    row: usize,
    text: Text80x25,
    background: Color16,
    foreground: Color16,
}

impl VgaTextScreen {
    pub fn new() -> Self {
        let text = Text80x25::new();
        text.enable_cursor();
        text.clear_screen();
        VgaTextScreen {
            col: 0,
            row: 0,
            text,
            background: DEFAULT_BACKGROUND,
            foreground: DEFAULT_FOREGROUND,
        }
    }

    fn new_line(&mut self) {
        self.col = 0;
        if self.row >= self.get_height() - 1 {
            for h in 1..self.get_height() {
                for w in 0..self.get_width() {
                    self.text
                        .write_character(w, h - 1, self.text.read_character(w, h))
                }
            }
            for w in 0..self.get_width() {
                self.text.write_character(
                    w,
                    self.get_height() - 1,
                    ScreenCharacter::new(
                        b' ',
                        TextModeColor::new(self.foreground, self.background),
                    ),
                )
            }
        } else {
            self.row += 1;
        }
    }
}

static PALETTE: [(Color16, Rgb); 16] = [
    (Color16::Black, Rgb { r: 0, g: 0, b: 0 }),
    (Color16::Blue, Rgb { r: 0, g: 0, b: 255 }),
    (Color16::Green, Rgb { r: 0, g: 255, b: 0 }),
    (
        Color16::Cyan,
        Rgb {
            r: 0,
            g: 255,
            b: 255,
        },
    ),
    (Color16::Red, Rgb { r: 255, g: 0, b: 0 }),
    (
        Color16::Magenta,
        Rgb {
            r: 255,
            g: 0,
            b: 255,
        },
    ),
    (
        Color16::Brown,
        Rgb {
            r: 150,
            g: 75,
            b: 0,
        },
    ),
    (
        Color16::LightGrey,
        Rgb {
            r: 24,
            g: 23,
            b: 26,
        },
    ),
    (
        Color16::DarkGrey,
        Rgb {
            r: 105,
            g: 105,
            b: 105,
        },
    ),
    (
        Color16::LightBlue,
        Rgb {
            r: 0,
            g: 78,
            b: 255,
        },
    ),
    (
        Color16::LightGreen,
        Rgb {
            r: 102,
            g: 255,
            b: 0,
        },
    ),
    (
        Color16::LightCyan,
        Rgb {
            r: 45,
            g: 253,
            b: 254,
        },
    ),
    (
        Color16::LightRed,
        Rgb {
            r: 170,
            g: 1,
            b: 20,
        },
    ),
    (
        Color16::Pink,
        Rgb {
            r: 255,
            g: 192,
            b: 203,
        },
    ),
    (
        Color16::Yellow,
        Rgb {
            r: 255,
            g: 255,
            b: 0,
        },
    ),
    (
        Color16::White,
        Rgb {
            r: 255,
            g: 255,
            b: 255,
        },
    ),
];

fn closest_color(color: Rgb) -> Color16 {
    PALETTE
        .iter()
        .min_by_key(|(_, current)| {
            let r_diff = color.r as i32 - current.r as i32;
            let g_diff = color.g as i32 - current.g as i32;
            let b_diff = color.b as i32 - current.b as i32;
            r_diff * r_diff + g_diff * g_diff + b_diff * b_diff
        })
        .unwrap()
        .0
}

impl TtyScreen for VgaTextScreen {
    fn set_title(&mut self, _title: &str) {}

    fn write(&mut self, c: char) {
        if c == '\n' {
            self.new_line();
            return;
        }
        if self.col >= self.get_width() {
            self.new_line();
        }
        self.text.write_character(
            self.col,
            self.row,
            ScreenCharacter::new(
                c as u8,
                TextModeColor::new(self.foreground, self.background),
            ),
        );
        self.col += 1;
        if self.col >= self.get_width() {
            self.text
                .set_cursor_position(0, (self.row + 1).min(self.get_height()));
        } else {
            self.text.set_cursor_position(self.col, self.row);
        }
    }

    fn erase(&mut self, row: usize, col: usize) {
        self.text.write_character(
            col,
            row,
            ScreenCharacter::new(
                b' ',
                TextModeColor::new(DEFAULT_FOREGROUND, DEFAULT_BACKGROUND),
            ),
        );
    }

    fn delete(&mut self, chars: usize) {
        let max_idx = self.get_height() * self.get_width();
        for row in self.row..self.get_height() {
            for col in self.col..self.get_width() {
                let idx = row * self.get_width() + col + chars;
                let char = if idx >= max_idx {
                    ScreenCharacter::new(
                        b' ',
                        TextModeColor::new(DEFAULT_FOREGROUND, DEFAULT_BACKGROUND),
                    )
                } else {
                    self.text
                        .read_character(idx % self.get_width(), idx / self.get_width())
                };
                self.text.write_character(col, row, char)
            }
        }
    }

    #[inline]
    fn set_row(&mut self, row: usize) {
        self.row = row;
        self.text.set_cursor_position(self.col, self.row);
    }

    #[inline]
    fn set_col(&mut self, col: usize) {
        self.col = col;
        self.text.set_cursor_position(self.col, self.row);
    }

    #[inline]
    fn get_row(&self) -> usize {
        self.row
    }

    #[inline]
    fn get_col(&self) -> usize {
        self.col
    }

    #[inline]
    fn get_width(&self) -> usize {
        80
    }

    #[inline]
    fn get_height(&self) -> usize {
        25
    }

    fn set_style(&mut self, shape: CursorStyle) {
        match shape.shape {
            CursorShape::Hidden => self.text.disable_cursor(),
            _ => self.text.enable_cursor(),
        }
    }

    fn clear_all(&mut self) {
        self.text.clear_screen();
        self.clear_history();
    }

    fn scroll_up(&mut self, rows: usize) {
        // fixme
    }

    fn scroll_down(&mut self, rows: usize) {
        // fixme
    }

    fn clear_history(&mut self) {
        // fixme
    }

    fn set_foreground(&mut self, color: Rgb) {
        self.foreground = closest_color(color);
    }

    fn set_background(&mut self, color: Rgb) {
        self.background = closest_color(color);
    }
}
