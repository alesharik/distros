#![no_std]

mod rdrand;

use crate::rdrand::HwRng;
use rand::RngCore;
use rand_pcg::Pcg64Mcg;
use x86_64::instructions::random::RdRand;

pub fn new_random() -> impl RngCore {
    HwRng::new(
        RdRand::new(),
        Pcg64Mcg::new(distros_timer::uptime().as_nanos() * 2000 / 3 * 13),
    )
}
