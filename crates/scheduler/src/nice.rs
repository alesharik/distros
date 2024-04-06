#[derive(Eq, PartialEq, Ord, PartialOrd, Copy, Clone, Debug)]
#[repr(transparent)]
pub struct NiceLevel(i8);

impl Default for NiceLevel {
    fn default() -> Self {
        NiceLevel::new(0)
    }
}

impl NiceLevel {
    pub const MIN: NiceLevel = NiceLevel(-20);
    pub const MAX: NiceLevel = NiceLevel(20);

    pub fn new(level: i8) -> NiceLevel {
        let l = NiceLevel(level);
        if l < Self::MIN {
            panic!("Level {:?} must be >= {:?}", l, Self::MIN);
        }
        if l > Self::MAX {
            panic!("Level {:?} must be >= {:?}", l, Self::MAX);
        }
        l
    }

    #[inline]
    pub fn level(&self) -> i8 {
        self.0
    }
}
