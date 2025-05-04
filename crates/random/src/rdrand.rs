use rand::RngCore;
use x86_64::instructions::random::RdRand;

pub struct HwRng<F: RngCore + Send + Sync + 'static> {
    rdrand: Option<RdRand>,
    fallback: F,
}

impl<F: RngCore + Send + Sync + 'static> HwRng<F> {
    pub fn new(rdrand: Option<RdRand>, fallback: F) -> Self {
        HwRng { rdrand, fallback }
    }

    #[inline(always)]
    fn fill_bytes_impl(&mut self, dest: &mut [u8]) {
        let mut left = dest;
        while left.len() >= 8 {
            let (l, r) = { left }.split_at_mut(8);
            left = r;
            let chunk: [u8; 8] = self.next_u64().to_le_bytes();
            l.copy_from_slice(&chunk);
        }
        let n = left.len();
        if n > 0 {
            let chunk: [u8; 8] = self.next_u64().to_le_bytes();
            left.copy_from_slice(&chunk[..n]);
        }
    }
}

impl<F: RngCore + Send + Sync + 'static> RngCore for HwRng<F> {
    fn next_u32(&mut self) -> u32 {
        self.rdrand
            .and_then(|s| s.get_u32())
            .unwrap_or_else(|| self.fallback.next_u32())
    }

    fn next_u64(&mut self) -> u64 {
        self.rdrand
            .and_then(|s| s.get_u64())
            .unwrap_or_else(|| self.fallback.next_u64())
    }

    fn fill_bytes(&mut self, dest: &mut [u8]) {
        self.fill_bytes_impl(dest)
    }
}
