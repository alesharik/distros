#![no_std]

use bootloader_api::info::{FrameBufferInfo, PixelFormat};
use core::ptr;
use spin::{Mutex, MutexGuard};
use framebuffer::{FrameBuffer, FrameBufferWrite, Rgb};

struct Inner {
    fb: &'static mut [u8],
    info: FrameBufferInfo,
}

impl Inner {
    pub fn write(&mut self, x: u16, y: u16, color: Rgb) {
        let pixel_offset = y as usize * self.info.stride + x as usize;
        let color = match self.info.pixel_format {
            PixelFormat::Rgb => [color.r, color.g, color.b, 0],
            PixelFormat::Bgr => [color.b, color.g, color.g, 0],
            PixelFormat::U8 => [if color.grayscale() > 200 { 0xf } else { 0 }, 0, 0, 0],
            other => {
                // set a supported (but invalid) pixel format before panicking to avoid a double
                // panic; it might not be readable though
                self.info.pixel_format = PixelFormat::Rgb;
                panic!("pixel format {:?} not supported in logger", other)
            }
        };
        let bytes_per_pixel = self.info.bytes_per_pixel;
        let byte_offset = pixel_offset * bytes_per_pixel;
        self.fb[(byte_offset)..(byte_offset + bytes_per_pixel)]
            .copy_from_slice(&color[..bytes_per_pixel]);
        let _ = unsafe { ptr::read_volatile(&self.fb[byte_offset]) };
    }

    fn clear(&mut self) {
        self.fb.fill(0);
    }
}

pub struct VesaFrameBuffer {
    inner: Mutex<Inner>,
    width: u16,
    height: u16,
}

impl VesaFrameBuffer {
    pub fn new(fb: bootloader_api::info::FrameBuffer) -> Self {
        let info = fb.info();
        VesaFrameBuffer {
            width: info.width as u16,
            height: info.height as u16,
            inner: Mutex::new(Inner {
                info,
                fb: fb.into_buffer(),
            })
        }
    }

    pub fn force_write(&self) -> FbWrite<'_> {
        if self.inner.is_locked() {
            unsafe { self.inner.force_unlock(); }
        }
        let inner = self.inner.lock();
        FbWrite { inner }
    }
}

impl FrameBuffer for VesaFrameBuffer {
    fn write(&self) -> FbWrite {
        let inner = self.inner.lock();
        FbWrite { inner }
    }

    #[inline]
    fn width(&self) -> u16 {
        self.width
    }

    #[inline]
    fn height(&self) -> u16 {
        self.height
    }
}

struct FbWrite<'a> {
    inner: MutexGuard<'a, Inner>,
}

impl<'a> FrameBufferWrite for FbWrite<'a> {
    fn write_pixel(&mut self, x: u16, y: u16, color: Rgb) {
        self.inner.write(x, y, color);
    }

    fn clear(&mut self) {
        self.inner.clear();
    }
}