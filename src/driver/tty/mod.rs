use crate::driver::tty::flow::{Stdin, StdinKeyboardConsumer, Stdout};
use crate::flow::{FlowManager, FlowManagerError};
use alloc::borrow::ToOwned;
use alloc::boxed::Box;
use alloc::string::{String, ToString};
use alloc::sync::Arc;
use alloc::vec::Vec;
use core::fmt::{Debug, Formatter};
use core::marker::PhantomData;
use hashbrown::HashMap;
use libkernel::flow::Message;
use spin::{Lazy, Mutex};
use vte::ansi::{
    Attr, CharsetIndex, ClearMode, Color, CursorShape, CursorStyle, Handler, LineClearMode, Mode,
    NamedColor, Rgb, StandardCharset, TabulationClearMode,
};

mod flow;
// mod vga;

trait TtyScreen {
    fn set_title(&mut self, title: &str);
    fn write(&mut self, c: char);
    /// reset char state
    fn erase(&mut self, row: usize, col: usize);
    /// delete char and move all all things to the left
    fn delete(&mut self, chars: usize);
    fn set_row(&mut self, row: usize);
    fn set_col(&mut self, col: usize);
    fn get_row(&self) -> usize;
    fn get_col(&self) -> usize;
    fn get_width(&self) -> usize;
    fn get_height(&self) -> usize;
    fn set_style(&mut self, shape: CursorStyle);
    fn scroll_up(&mut self, rows: usize);
    fn scroll_down(&mut self, rows: usize);
    fn clear_all(&mut self);
    fn clear_history(&mut self);
    fn set_foreground(&mut self, color: Rgb);
    fn set_background(&mut self, color: Rgb);
}

trait TtyWriter {
    fn write_back(&mut self, s: &str);
}

struct DefaultHandler<S: TtyScreen, W: TtyWriter> {
    writer: S,
    titles: Vec<String>,
    saved_position: Option<(usize, usize)>,
    title: String,
    colors: HashMap<usize, Rgb>,
    writer_type: PhantomData<W>,
}

impl<S: TtyScreen, W: TtyWriter> DefaultHandler<S, W> {
    fn new(screen: S) -> DefaultHandler<S, W> {
        DefaultHandler {
            writer: screen,
            titles: Vec::new(),
            title: "".to_owned(),
            saved_position: None,
            colors: HashMap::new(),
            writer_type: PhantomData::default(),
        }
    }

    fn color_to_rgb(&self, color: Color) -> Rgb {
        match color {
            Color::Indexed(idx) => self.colors[&(idx as usize)],
            Color::Spec(rgb) => rgb,
            Color::Named(named) => match named {
                NamedColor::White => Rgb {
                    r: 255,
                    g: 255,
                    b: 255,
                },
                NamedColor::Black => Rgb { r: 0, g: 0, b: 0 },
                NamedColor::Background => Rgb { r: 0, g: 0, b: 0 },
                NamedColor::Foreground => Rgb {
                    r: 255,
                    g: 255,
                    b: 255,
                },
                NamedColor::Green => Rgb { r: 0, g: 255, b: 0 },
                NamedColor::Blue => Rgb { r: 0, g: 0, b: 255 },
                NamedColor::Yellow => Rgb {
                    r: 255,
                    g: 255,
                    b: 0,
                },
                NamedColor::Red => Rgb { r: 255, g: 0, b: 0 },
                NamedColor::Magenta => Rgb {
                    r: 255,
                    g: 0,
                    b: 255,
                },
                NamedColor::Cyan => Rgb {
                    r: 0,
                    g: 255,
                    b: 255,
                },
                NamedColor::Cursor => Rgb {
                    r: 255,
                    g: 255,
                    b: 255,
                },
                NamedColor::BrightBlack => Rgb {
                    r: 24,
                    g: 23,
                    b: 26,
                },
                NamedColor::BrightBlue => Rgb {
                    r: 0,
                    g: 78,
                    b: 255,
                },
                NamedColor::BrightCyan => Rgb {
                    r: 45,
                    g: 253,
                    b: 254,
                },
                NamedColor::BrightForeground => Rgb {
                    r: 253,
                    g: 254,
                    b: 255,
                },
                NamedColor::BrightGreen => Rgb {
                    r: 102,
                    g: 255,
                    b: 0,
                },
                NamedColor::BrightMagenta => Rgb {
                    r: 255,
                    g: 8,
                    b: 232,
                },
                NamedColor::BrightRed => Rgb {
                    r: 170,
                    g: 1,
                    b: 20,
                },
                NamedColor::BrightWhite => Rgb {
                    r: 253,
                    g: 254,
                    b: 255,
                },
                NamedColor::BrightYellow => Rgb {
                    r: 255,
                    g: 255,
                    b: 237,
                },
                NamedColor::DimBlack => Rgb {
                    r: 105,
                    g: 105,
                    b: 105,
                },
                NamedColor::DimBlue => Rgb {
                    r: 0,
                    g: 86,
                    b: 161,
                },
                NamedColor::DimCyan => Rgb {
                    r: 51,
                    g: 85,
                    b: 102,
                },
                NamedColor::DimForeground => Rgb {
                    r: 150,
                    g: 160,
                    b: 170,
                },
                NamedColor::DimGreen => Rgb {
                    r: 68,
                    g: 85,
                    b: 68,
                },
                NamedColor::DimMagenta => Rgb {
                    r: 85,
                    g: 68,
                    b: 102,
                },
                NamedColor::DimRed => Rgb {
                    r: 85,
                    g: 51,
                    b: 68,
                },
                NamedColor::DimWhite => Rgb {
                    r: 150,
                    g: 160,
                    b: 170,
                },
                NamedColor::DimYellow => Rgb {
                    r: 85,
                    g: 80,
                    b: 69,
                },
            },
        }
    }
}

// reference impl - https://github.com/alacritty/alacritty/blob/3c61e075fef7b02ae0d043e4a4e664b8bc7221e9/alacritty_terminal/src/term/mod.rs
//noinspection RsSortImplTraitMembers
impl<S: TtyScreen, W: TtyWriter> Handler<W> for DefaultHandler<S, W> {
    #[inline]
    fn set_title_utf(&mut self, title: Option<String>) {
        let x = title.unwrap_or_else(|| "".to_owned());
        self.writer.set_title(&x);
        self.title = x;
    }

    #[inline]
    fn push_title(&mut self) {
        self.titles.push(self.title.clone());
        self.set_title_utf(None);
    }

    #[inline]
    fn pop_title(&mut self) {
        if let Some(title) = self.titles.pop() {
            self.set_title_utf(Some(title));
        }
    }

    #[inline]
    fn save_cursor_position(&mut self) {
        self.saved_position = Some((self.writer.get_row(), self.writer.get_col()))
    }

    #[inline]
    fn restore_cursor_position(&mut self) {
        if let Some(pos) = self.saved_position.take() {
            self.writer.set_row(pos.0);
            self.writer.set_col(pos.1);
        }
    }

    #[inline]
    fn input(&mut self, c: char) {
        self.writer.write(c)
    }

    #[inline]
    fn goto(&mut self, row: usize, col: usize) {
        self.writer.set_row(row);
        self.writer.set_col(col);
    }

    #[inline]
    fn goto_line(&mut self, row: usize) {
        self.writer.set_row(row);
    }

    #[inline]
    fn goto_col(&mut self, col: usize) {
        self.writer.set_col(col);
    }

    #[inline]
    fn insert_blank(&mut self, col: usize) {
        self.writer.set_col(col);
        for _ in col..self.writer.get_width() {
            self.input(' ');
        }
    }

    #[inline]
    fn put_tab(&mut self, count: i64) {
        for _ in 0..count * 4 {
            self.input(' ');
        }
    }

    #[inline]
    fn move_up(&mut self, row: usize) {
        let current_row = self.writer.get_row();
        if current_row >= row {
            self.writer.set_row(current_row - row);
        } else {
            let delta = row - current_row;
            self.writer.set_row(0);
            self.writer.scroll_up(delta);
        }
    }

    #[inline]
    fn move_down(&mut self, row: usize) {
        let current_row = self.writer.get_row();
        if current_row + row < self.writer.get_height() {
            self.writer.set_row(row + current_row)
        } else {
            let delta = current_row + row - self.writer.get_height();
            self.writer.set_row(self.writer.get_height());
            self.writer.scroll_down(delta);
        }
    }

    #[inline]
    fn move_forward(&mut self, col: usize) {
        let current_col = self.writer.get_col();
        if current_col + col < self.writer.get_width() {
            self.writer.set_col(current_col + col);
        } else {
            self.writer.set_col(self.writer.get_width() - 1);
        }
    }

    #[inline]
    fn move_backward(&mut self, col: usize) {
        let current_col = self.writer.get_col();
        if current_col >= col {
            self.writer.set_col(current_col - col);
        } else {
            self.writer.set_col(0);
        }
    }

    #[inline]
    fn move_down_and_cr(&mut self, row: usize) {
        self.move_down(row);
        self.writer.set_col(0);
    }

    #[inline]
    fn move_up_and_cr(&mut self, row: usize) {
        self.move_up(row);
        self.writer.set_col(0);
    }

    #[inline]
    fn carriage_return(&mut self) {
        self.input('\n');
    }

    #[inline]
    fn linefeed(&mut self) {
        self.input('\n');
    }

    #[inline]
    fn newline(&mut self) {
        self.input('\n');
    }

    #[inline]
    fn set_cursor_style(&mut self, style: Option<CursorStyle>) {
        let style = style.unwrap_or(CursorStyle {
            blinking: true,
            shape: CursorShape::Block,
        });
        self.writer.set_style(style);
    }

    #[inline]
    fn set_cursor_shape(&mut self, shape: CursorShape) {
        self.writer.set_style(CursorStyle {
            blinking: true,
            shape,
        })
    }

    #[inline]
    fn backspace(&mut self) {
        if self.writer.get_col() == 0 {
            self.writer.set_col(self.writer.get_width() - 1);
            if self.writer.get_row() > 0 {
                self.writer.set_row(self.writer.get_row() - 1);
            } else {
                return;
            }
        } else {
            self.writer.set_col(self.writer.get_col() - 1);
        }
        self.writer.delete(1);
    }

    #[inline]
    fn delete_lines(&mut self, count: usize) {
        for row in
            self.writer.get_row()..self.writer.get_height().min(self.writer.get_row() + count)
        {
            for col in 0..self.writer.get_width() {
                self.writer.erase(row, col);
            }
        }
    }

    #[inline]
    fn erase_chars(&mut self, count: usize) {
        for col in self.writer.get_col()..self.writer.get_width().min(self.writer.get_col() + count)
        {
            self.writer.erase(self.writer.get_row(), col);
        }
    }

    #[inline]
    fn delete_chars(&mut self, size: usize) {
        self.writer.delete(size)
    }

    #[inline]
    fn clear_line(&mut self, mode: LineClearMode) {
        match mode {
            LineClearMode::Left => {
                for col in 0..self.writer.get_col() {
                    self.writer.erase(self.writer.get_row(), col)
                }
            }
            LineClearMode::Right => {
                for col in (self.writer.get_col() + 1)..self.writer.get_width() {
                    self.writer.erase(self.writer.get_row(), col)
                }
            }
            LineClearMode::All => self.delete_lines(1),
        }
    }

    #[inline]
    fn clear_screen(&mut self, mode: ClearMode) {
        match mode {
            ClearMode::Saved => self.writer.clear_history(),
            ClearMode::All => self.writer.clear_all(),
            ClearMode::Above => {
                self.writer.clear_history();
                for row in 0..self.writer.get_row() {
                    for col in 0..self.writer.get_width() {
                        self.writer.erase(row, col)
                    }
                }
            }
            ClearMode::Below => {
                for row in self.writer.get_row()..self.writer.get_height() {
                    for col in 0..self.writer.get_width() {
                        self.writer.erase(row, col)
                    }
                }
            }
        }
    }

    #[inline]
    fn scroll_up(&mut self, count: usize) {
        self.writer.scroll_up(count);
    }

    #[inline]
    fn scroll_down(&mut self, count: usize) {
        self.writer.scroll_down(count);
    }

    #[inline]
    fn insert_blank_lines(&mut self, count: usize) {
        for _ in 0..count {
            self.input('\n');
        }
    }

    #[inline]
    fn move_backward_tabs(&mut self, count: i64) {
        self.move_backward((count * 4) as usize);
    }

    #[inline]
    fn move_forward_tabs(&mut self, count: i64) {
        self.move_forward((count * 4) as usize)
    }

    #[inline]
    fn reset_state(&mut self) {
        self.writer.clear_all();
        self.saved_position = None;
        self.titles.clear();
        self.set_title_utf(None);
        self.colors.clear();
    }

    #[inline]
    fn reverse_index(&mut self) {
        self.move_up(1);
    }

    fn terminal_attribute(&mut self, attr: Attr) {
        match attr {
            Attr::Foreground(color) => self.writer.set_foreground(self.color_to_rgb(color)),
            Attr::Background(color) => self.writer.set_background(self.color_to_rgb(color)),
            Attr::Reset => {
                self.writer
                    .set_foreground(self.color_to_rgb(Color::Named(NamedColor::White)));
                self.writer
                    .set_background(self.color_to_rgb(Color::Named(NamedColor::Black)));
            }
            _ => debug!("Attr not supported {:?}", attr),
        }
    }

    #[inline]
    fn set_color(&mut self, id: usize, color: Rgb) {
        self.colors.insert(id, color);
    }

    #[inline]
    fn reset_color(&mut self, id: usize) {
        self.colors.remove(&id);
    }

    #[inline]
    fn identify_terminal(&mut self, writer: &mut W, intermediate: Option<char>) {
        match intermediate {
            None => {
                writer.write_back("\x1b[?6c");
            }
            Some('>') => {
                let version = "1.0";
                let text = format!("\x1b[>0;{};1c", version);
                writer.write_back(&text);
            }
            _ => debug!("Unsupported device attributes intermediate"),
        }
    }

    #[inline]
    fn device_status(&mut self, writer: &mut W, arg: usize) {
        match arg {
            5 => writer.write_back("\x1b[0n"),
            6 => {
                writer.write_back("\x1b[");
                writer.write_back(&(self.writer.get_row() + 1).to_string());
                writer.write_back(";");
                writer.write_back(&(self.writer.get_col() + 1).to_string());
                writer.write_back("R");
            }
            _ => debug!("unknown device status query: {}", arg),
        };
    }

    fn text_area_size_pixels(&mut self, writer: &mut W) {
        let width = 5 * self.writer.get_width();
        let height = 8 * self.writer.get_height();
        let text = format!("\x1b[4;{};{}t", height, width);
        writer.write_back(&text);
    }

    fn text_area_size_chars(&mut self, writer: &mut W) {
        let text = format!(
            "\x1b[8;{};{}t",
            self.writer.get_height(),
            self.writer.get_width()
        );
        writer.write_back(&text);
    }

    #[inline]
    fn bell(&mut self) {}

    #[inline]
    fn set_active_charset(&mut self, _: CharsetIndex) {}

    #[inline]
    fn configure_charset(&mut self, _: CharsetIndex, charset: StandardCharset) {
        if charset != StandardCharset::Ascii {
            debug!("Charset {:?} not supported", charset)
        }
    }

    fn set_keypad_application_mode(&mut self) {
        todo!()
    }

    fn unset_keypad_application_mode(&mut self) {
        todo!()
    }

    fn set_mode(&mut self, _mode: Mode) {
        todo!()
    }

    fn unset_mode(&mut self, _: Mode) {
        todo!()
    }

    fn dynamic_color_sequence(&mut self, _: &mut W, _: u8, _: usize, _: &str) {
        todo!()
    }

    fn set_scrolling_region(&mut self, _top: usize, _bottom: Option<usize>) {
        todo!()
    }

    fn substitute(&mut self) {
        todo!()
    }

    fn decaln(&mut self) {
        todo!()
    }

    fn set_horizontal_tabstop(&mut self) {
        todo!()
    }

    fn clear_tabs(&mut self, _mode: TabulationClearMode) {
        todo!()
    }

    fn clipboard_store(&mut self, _: u8, _: &[u8]) {
        todo!()
    }

    fn clipboard_load(&mut self, _: u8, _: &str) {
        todo!()
    }
}

#[repr(transparent)]
pub struct TtyMessage(String);

impl TtyMessage {
    pub fn new(message: &str) -> Self {
        TtyMessage(message.to_owned())
    }
}

impl Message for TtyMessage {}

impl Debug for TtyMessage {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl ToString for TtyMessage {
    fn to_string(&self) -> String {
        self.0.to_owned()
    }
}

static VGA_STDIN: Lazy<Arc<Mutex<Stdin>>> = Lazy::new(|| Arc::new(Mutex::new(Stdin::new())));

pub fn init() -> Result<(), FlowManagerError> {
    debug!("Setting up TTY devices");
    // let stdout = Stdout::new(
    //     DefaultHandler::new(vga::VgaTextScreen::new()),
    //     VGA_STDIN.clone(),
    // );
    // let sub = FlowManager::subscribe(
    //     "/dev/ps2/keyboard",
    //     Box::new(StdinKeyboardConsumer::new(VGA_STDIN.clone())),
    // )?;
    // core::mem::forget(sub); // never unsubscribe from device
    // FlowManager::register_endpoint(
    //     "/dev/tty/vga",
    //     VGA_STDIN.clone(),
    //     Some(Arc::new(Mutex::new(stdout))),
    // )
    // .unwrap();
    debug!("VGA TTY device set up");
    Ok(())
}
