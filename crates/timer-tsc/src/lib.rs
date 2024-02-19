#![no_std]

use core::arch::x86_64::_rdtsc;
use core::time::Duration;

const REQUIRED_CALIBRATION_SAMPLES: usize = 64;
static mut SAMPLES: [u64; REQUIRED_CALIBRATION_SAMPLES] = [0; REQUIRED_CALIBRATION_SAMPLES];
static mut COLLECTED_SAMPLES: usize = 0;
static mut CALIB_FREQ: Duration = Duration::from_secs(0);
static mut CALIB_MEAN: u64 = 0;

pub fn tsc_calibration_frequency(freq: Duration) {
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
            let mut deltas = [0u64; REQUIRED_CALIBRATION_SAMPLES - 1];
            let mut last: Option<u64> = None;
            for (i, x) in SAMPLES.iter().enumerate() {
                if let Some(last) = last.take() {
                    deltas[i - 1] = *x - last;
                }
                last = Some(*x);
            }
            let mean_delta = deltas.iter().sum::<u64>() / deltas.len() as u64;
            CALIB_MEAN = mean_delta;
        }
    }
}

#[inline]
pub fn uptime() -> Duration {
    unsafe {
        if CALIB_MEAN == 0 {
            return Duration::ZERO;
        }
        let tsc = _rdtsc();
        let nanos = tsc * CALIB_FREQ.as_nanos() as u64 / CALIB_MEAN;
        Duration::from_nanos(nanos)
    }
}

pub fn sleep(duration: Duration) {
    let now = uptime();
    while uptime().max(now) - now < duration {
        x86_64::instructions::hlt();
    }
}
