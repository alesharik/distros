#![no_std]

use core::arch::asm;
use distros_cpuid::FpuInfo;
use log::{debug, info, warn};
use x86_64::registers::control::{Cr0, Cr0Flags, Cr4, Cr4Flags};
use x86_64::registers::xcontrol::{XCr0, XCr0Flags};

#[derive(Copy, Clone, Eq, PartialEq)]
enum SaveState {
    None,
    Fxsave,
    Xsave,
}

static mut SAVE_STATE: SaveState = SaveState::None;

fn init_sse(info: &FpuInfo) -> bool {
    if !info.sse {
        warn!("CPU does not have SSE support");
        return false;
    }
    if !info.fxsave_fxstor {
        warn!("CPU does not have fxsave/fxstor support");
        return false;
    }
    unsafe {
        Cr0::update(|flags| flags.set(Cr0Flags::EMULATE_COPROCESSOR, false));
        Cr0::update(|flags| flags.set(Cr0Flags::MONITOR_COPROCESSOR, true));
        Cr4::update(|flags| flags.set(Cr4Flags::OSFXSR, true));
        Cr4::update(|flags| flags.set(Cr4Flags::OSXMMEXCPT_ENABLE, true));
    }
    info!("SSE enabled");
    true
}

fn init_avx(info: &FpuInfo) -> bool {
    if !info.avx {
        warn!("CPU does not have AVX support");
        return false;
    }
    if !info.xsave {
        warn!("CPU does not hve xsave support");
        return false;
    }

    unsafe {
        let mut flags = XCr0::read();
        flags.set(XCr0Flags::AVX, true);
        flags.set(XCr0Flags::SSE, true);
        flags.set(XCr0Flags::X87, true);
        XCr0::write(flags);
    }
    info!("AVX enabled");
    true
}

fn check_fpu(info: &FpuInfo) -> bool {
    if !info.fpu {
        info!("CPU does not have FPU");
        return false;
    }
    unsafe {
        Cr0::update(|flags| flags.set(Cr0Flags::TASK_SWITCHED, false));
        asm!("fninit");
        let status: u16;
        asm!(
        "mov ax, ~0",
        "fnstsw ax",
        out("ax") status
        );
        return if status == 0 {
            info!("FPU ready");
            true
        } else {
            warn!("FPU failed to set status");
            false
        };
    }
}

pub fn init() {
    info!("Starting FPU");
    let info = FpuInfo::load();
    debug!("CPU info: {:?}", &info);
    if init_sse(&info) && check_fpu(&info) {
        let state = if init_avx(&info) {
            SaveState::Xsave
        } else {
            SaveState::Fxsave
        };
        unsafe { SAVE_STATE = state }
    }
}

#[repr(C, align(64))]
#[derive(Clone)]
pub struct FpuState {
    data: [u8; 2584],
}

impl FpuState {
    pub const fn new() -> Self {
        FpuState { data: [0u8; 2584] }
    }
    pub unsafe fn save(&mut self) {
        match SAVE_STATE {
            SaveState::Fxsave => self.fxsave(),
            SaveState::Xsave => self.xsave(),
            SaveState::None => {}
        }
    }

    pub unsafe fn restore(&self) {
        match SAVE_STATE {
            SaveState::Fxsave => self.fxrstor(),
            SaveState::Xsave => self.xrstor(),
            SaveState::None => {}
        }
    }

    #[inline(never)]
    unsafe fn fxsave(&mut self) {
        asm!(
        "mov eax, ~0",
        "mov edx, ~0",
        "fxsave [{}]",
        in(reg) self,
        out("eax") _,
        out("edx") _,
        )
    }

    #[inline(never)]
    unsafe fn xsave(&mut self) {
        asm!(
        "mov eax, ~0",
        "mov edx, ~0",
        "xsave [{}]",
        in(reg) self,
        out("eax") _,
        out("edx") _,
        )
    }

    #[inline(never)]
    unsafe fn xrstor(&self) {
        asm!(
        "mov eax, ~0",
        "mov edx, ~0",
        "xrstor [{}]",
        in(reg) self,
        out("eax") _,
        out("edx") _,
        )
    }

    #[inline(never)]
    unsafe fn fxrstor(&self) {
        asm!(
        "mov eax, ~0",
        "mov edx, ~0",
        "fxrstor [{}]",
        in(reg) self,
        out("eax") _,
        out("edx") _,
        )
    }
}
