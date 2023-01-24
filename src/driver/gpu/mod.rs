mod vesa;

pub use vesa::{VesaGpu, VesaFrameBuffer};

pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8
}

pub trait FrameBuffer {
    fn write_pixel(&mut self, x: usize, y: usize, color: Rgb);

    fn write_monochrome_pixels(&mut self, x: usize, y: usize, pixels: &[&[u8]]);

    fn width(&self) -> usize;

    fn height(&self) -> usize;

    fn clear(&self);
}

pub trait Gpu {
    type FrameBuffer: FrameBuffer;

    fn new_framebuffer(&self) -> Self::FrameBuffer;

    unsafe fn new_framebuffer_priority(&self) -> Self::FrameBuffer;
}