#![no_std]

use core::arch::x86_64::_rdtsc;
use core::time::Duration;
use log::debug;

const REQUIRED_CALIBRATION_SAMPLES: usize = 64;
static mut SAMPLES: [u64; REQUIRED_CALIBRATION_SAMPLES] = [0; REQUIRED_CALIBRATION_SAMPLES];
static mut COLLECTED_SAMPLES: usize = 0;
static mut CALIB_FREQ: usize = 0;
static mut CALIB_MS: u64 = 0;

pub fn tsc_calibration_frequency(freq: usize) {
    unsafe {
        CALIB_FREQ = freq;
    }
}

pub fn tsc_calibration_sample() {
    unsafe {
        if COLLECTED_SAMPLES == REQUIRED_CALIBRATION_SAMPLES {
            return;
        }
        SAMPLES[COLLECTED_SAMPLES] = unsafe { _rdtsc() };
        COLLECTED_SAMPLES += 1;
        if COLLECTED_SAMPLES == REQUIRED_CALIBRATION_SAMPLES {
            let mean_delta = SAMPLES.iter().sum::<u64>()
                / REQUIRED_CALIBRATION_SAMPLES as u64
                / CALIB_FREQ as u64;
            CALIB_MS = mean_delta;
        }
    }
}

#[inline]
pub fn uptime() -> Duration {
    unsafe {
        if CALIB_MS == 0 {
            return Duration::ZERO;
        }
        let tsc = _rdtsc();
        let nanos = tsc / CALIB_MS * 1_000_000;
        Duration::from_nanos(nanos)
    }
}

pub fn sleep(duration: Duration) {
    let now = uptime();
    while uptime().max(now) - now < duration {
        x86_64::instructions::hlt();
    }
}
