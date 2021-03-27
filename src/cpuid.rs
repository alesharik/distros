use raw_cpuid::CpuId;
use spin::Mutex;
use crate::kblog;

lazy_static!(
    static ref CPUID: Mutex<Option<CpuId>> = Mutex::new(Option::None);
);

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