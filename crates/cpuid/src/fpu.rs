use crate::get_feature_info;
use raw_cpuid::FeatureInfo;

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

    pub fn load() -> FpuInfo {
        Self::from_feature_info(&get_feature_info())
    }
}
