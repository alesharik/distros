#![no_std]

mod fpu;

use log::info;
use raw_cpuid::{CpuId, CpuIdReaderNative, FeatureInfo, ProcessorFrequencyInfo};

pub use fpu::FpuInfo;

static mut CPUID: Option<CpuId<CpuIdReaderNative>> = None;

pub fn load() {
    unsafe {
        CPUID = Some(CpuId::new());
    }
    info!("CPUID set");
}

pub fn get_feature_info() -> FeatureInfo {
    unsafe {
        CPUID
            .as_ref()
            .expect("CPUID should be loaded first")
            .get_feature_info()
            .expect("CPUID does not have feature infos")
    }
}

pub fn get_processor_frequency_info() -> Option<ProcessorFrequencyInfo> {
    unsafe {
        CPUID
            .as_ref()
            .expect("CPUID should be loaded first")
            .get_processor_frequency_info()
    }
}
