use bootloader_api::info::{FrameBufferInfo, PixelFormat};
use core::{fmt, ptr};
use spin::{Mutex, RwLock};
use crate::driver::gpu::{FrameBuffer, Gpu, Rgb};

lazy_static!(
    static ref FRAMEBUFFER: Mutex<Option<FrameBufferInner>> = Mutex::new(None);
);

struct FrameBufferInner {
    framebuffer: &'static mut [u8],
    info: FrameBufferInfo,
}

impl FrameBufferInner {
    pub fn write(&mut self, x: usize, y: usize, color: Rgb) {
        let pixel_offset = y * self.info.stride + x;
        let color = match self.info.pixel_format {
            PixelFormat::Rgb => [color.r, color.g, color.b, 0],
            PixelFormat::Bgr => [color.b, color.g, color.g, 0],
            PixelFormat::U8 => [if (color.r as usize + color.g as usize + color.b as usize) / 3 > 200 { 0xf } else { 0 }, 0, 0, 0],
            other => {
                // set a supported (but invalid) pixel format before panicking to avoid a double
                // panic; it might not be readable though
                self.info.pixel_format = PixelFormat::Rgb;
                panic!("pixel format {:?} not supported in logger", other)
            }
        };
        let bytes_per_pixel = self.info.bytes_per_pixel;
        let byte_offset = pixel_offset * bytes_per_pixel;
        self.framebuffer[byte_offset..(byte_offset + bytes_per_pixel)]
            .copy_from_slice(&color[..bytes_per_pixel]);
        let _ = unsafe { ptr::read_volatile(&self.framebuffer[byte_offset]) };
    }

    fn clear(&mut self) {
        self.framebuffer.fill(0);
    }
}

/// Allows logging text to a pixel-based framebuffer.
pub struct VesaFrameBuffer {
    info: FrameBufferInfo,
    priority: bool,
}

impl VesaFrameBuffer {
    /// Creates a new logger that uses the given framebuffer.
    fn new(info: FrameBufferInfo, priority: bool) -> Self {
        Self {
            info,
            priority,
        }
    }
}

impl FrameBuffer for VesaFrameBuffer {
    fn write_pixel(&mut self, x: usize, y: usize, color: Rgb) {
        if self.priority {
            unsafe { FRAMEBUFFER.force_unlock() };
        }
        let mut fb = FRAMEBUFFER.lock();
        if let Some(fb) = fb.as_mut() {
            fb.write(x, y, color);
        }
    }

    fn write_monochrome_pixels(&mut self, x_pos: usize, y_pos: usize, pixels: &[&[u8]]) {
        if self.priority {
            unsafe { FRAMEBUFFER.force_unlock() };
        }
        let mut fb = FRAMEBUFFER.lock();
        if let Some(fb) = fb.as_mut() {
            for (y, row) in pixels.iter().enumerate() {
                for (x, byte) in row.iter().enumerate() {
                    let rgb = Rgb {
                        r: *byte,
                        g: *byte,
                        b: *byte / 2
                    };
                    fb.write(x_pos + x, y_pos + y, rgb);
                }
            }
        }
    }

    fn width(&self) -> usize {
        self.info.width
    }

    fn height(&self) -> usize {
        self.info.height
    }

    fn clear(&self) {
        if self.priority {
            unsafe { FRAMEBUFFER.force_unlock() };
        }
        let mut fb = FRAMEBUFFER.lock();
        if let Some(fb) = fb.as_mut() {
            fb.clear();
        }
    }
}

pub struct VesaGpu {
    info: FrameBufferInfo
}

impl VesaGpu {
    pub fn new(fb: &'static mut [u8], info: FrameBufferInfo) -> Self {
        let mut framebuffer = FRAMEBUFFER.lock();
        *framebuffer = Some(FrameBufferInner {
            framebuffer: fb,
            info
        });
        VesaGpu {
            info
        }
    }
}

impl Gpu for VesaGpu {
    type FrameBuffer = VesaFrameBuffer;

    fn new_framebuffer(&self) -> Self::FrameBuffer {
        VesaFrameBuffer::new(self.info, false)
    }

    unsafe fn new_framebuffer_priority(&self) -> Self::FrameBuffer {
        VesaFrameBuffer::new(self.info, true)
    }
}