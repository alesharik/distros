use x86_64::instructions::random::RdRand;
use rand_pcg::Pcg64Mcg;
use rand::{RngCore, Error};
use alloc::boxed::Box;
use crate::interrupts;

struct HwRng {
    rdrand: RdRand,
    fallback: Pcg64Mcg
}

impl HwRng {
    fn new(rdrand: RdRand) -> Self {
        HwRng {
            rdrand,
            fallback: Pcg64Mcg::new((interrupts::now() as u128) * 2000 / 3 * 13)
        }
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

impl RngCore for HwRng {
    fn next_u32(&mut self) -> u32 {
        self.rdrand.get_u32().unwrap_or(self.fallback.next_u32())
    }

    fn next_u64(&mut self) -> u64 {
        self.rdrand.get_u64().unwrap_or(self.fallback.next_u64())
    }

    fn fill_bytes(&mut self, dest: &mut [u8]) {
        self.fill_bytes_impl(dest)
    }

    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), Error> {
        self.fill_bytes(dest);
        Ok(())
    }

}

pub fn rng() -> Box<dyn RngCore> {
    RdRand::new()
        .map(|rdrand| Box::new(HwRng::new(rdrand)) as Box<dyn RngCore>)
        .unwrap_or_else(|| Box::new(Pcg64Mcg::new((interrupts::now() as u128) * 2000 / 3 * 13)))
}