#![no_std]

mod rgb;

pub use rgb::Rgb;

pub trait FrameBufferWrite: Send + Sync {
    fn write_pixel(&mut self, x: u16, y: u16, color: Rgb);

    fn write_monochrome_pixels(&mut self, x_pos: u16, y_pos: u16, pixels: &[&[u8]]) {
        for (y, row) in pixels.iter().enumerate() {
            for (x, byte) in row.iter().enumerate() {
                let rgb = Rgb::from_monochrome(*byte);
                self.write_pixel(x_pos + x as u16, y_pos + y as u16, rgb);
            }
        }
    }

    fn clear(&mut self);
}

pub trait FrameBuffer: Send + Sync {
    fn write(&self) -> impl FrameBufferWrite;

    fn width(&self) -> u16;

    fn height(&self) -> u16;
}