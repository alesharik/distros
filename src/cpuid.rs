use raw_cpuid::{CpuId, FeatureInfo};
use spin::Mutex;
use crate::kblog;

lazy_static!(
    static ref CPUID: Mutex<Option<CpuId>> = Mutex::new(Option::None);
);

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct FpuInfo {
    pub sse: bool,
    pub sse2: bool,
    pub sse3: bool,
    pub sse4_1: bool,
    pub sse4_2: bool,
    pub ssse3: bool,
    pub avx: bool,
    pub fxsave_fxstor: bool,
    pub xsave: bool,
    pub fma: bool,
    pub fpu: bool,
}

impl FpuInfo {
    fn from_feature_info(info: &FeatureInfo) -> Self {
        FpuInfo {
            sse: info.has_sse(),
            sse2: info.has_sse2(),
            sse3: info.has_sse3(),
            sse4_1: info.has_sse41(),
            sse4_2: info.has_sse42(),
            ssse3: info.has_ssse3(),
            avx: info.has_avx(),
            fxsave_fxstor: info.has_fxsave_fxstor(),
            xsave: info.has_xsave(),
            fma: info.has_fma(),
            fpu: info.has_fpu(),
        }
    }
}

pub fn init_cpuid() {
    let mut cpuid = CPUID.lock();
    *cpuid = Option::Some(CpuId::new());
    kblog!("CPUID", "CPUID set");
}

pub fn has_apic() -> bool {
    let guard = CPUID.lock();
    let cpuid = guard.as_ref();
    if let Some(cpuid) = cpuid {
        cpuid.get_feature_info().expect("Failed to get feature info").has_apic()
    } else {
        panic!("CPUID not initialized")
    }
}

pub fn get_fpu_info() -> FpuInfo {
    let guard = CPUID.lock();
    let cpuid = guard.as_ref();
    if let Some(cpuid) = cpuid {
        let info = cpuid.get_feature_info().expect("Failed to get feature info");
        FpuInfo::from_feature_info(&info)
    } else {
        panic!("CPUID not initialized")
    }
}
