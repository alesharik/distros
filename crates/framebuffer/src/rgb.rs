#[derive(Eq, PartialEq, Clone, Copy, Ord, PartialOrd)]
pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Rgb {
    #[inline]
    pub fn new(r: u8, g: u8, b: u8) -> Rgb {
        Rgb { r, g, b }
    }

    #[inline]
    pub fn from_monochrome(mono: u8) -> Rgb {
        Rgb {
            r: mono,
            g: mono,
            b: mono / 2,
        }
    }

    #[inline]
    pub fn grayscale(&self) -> u8 {
        ((self.r as usize + self.g as usize + self.b as usize) / 3) as u8
    }
}
