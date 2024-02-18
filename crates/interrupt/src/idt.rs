use crate::nmi::nmi_status;
use crate::{gdt, InterruptId};
use fixedbitset::FixedBitSet;
use lazy_static::lazy_static;
use log::{error, info};
use spin::mutex::Mutex;
use x86_64::structures::idt::{
    HandlerFunc, InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode,
};

lazy_static! {
    static ref IDT: Mutex<InterruptDescriptorTable> = Mutex::new({
        let mut idt = InterruptDescriptorTable::new();
        idt.divide_error.set_handler_fn(divide_error_handler);
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        idt.page_fault.set_handler_fn(page_fault_handler);
        idt.overflow.set_handler_fn(overflow_handler);
        idt.bound_range_exceeded.set_handler_fn(bound_handler);
        idt.invalid_opcode.set_handler_fn(invalid_opcode);
        idt.device_not_available
            .set_handler_fn(device_not_available);
        idt.non_maskable_interrupt.set_handler_fn(nmi_handler);
        idt.simd_floating_point.set_handler_fn(fpa_handler);
        idt.machine_check.set_handler_fn(machine_check_handler);
        idt.general_protection_fault
            .set_handler_fn(general_protection_fault);
        idt.segment_not_present
            .set_handler_fn(segment_not_present_handler);
        unsafe {
            idt.double_fault
                .set_handler_fn(double_fault_handler)
                .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
        }
        idt
    });
    static ref SET_INTS: Mutex<FixedBitSet> = Mutex::new(FixedBitSet::with_capacity(256));
}

pub fn init_idt() {
    let guard = IDT.lock();
    unsafe {
        guard.load_unsafe();
    }
    info!("IDT table loaded");
}

pub enum OverrideMode {
    Override,
    Panic,
}

pub fn set_handler(int: InterruptId, func: HandlerFunc, override_mode: OverrideMode) {
    let mut idt = IDT.lock();
    let mut set_ints = SET_INTS.lock();
    if set_ints.contains(int.int()) {
        match override_mode {
            OverrideMode::Override => {}
            OverrideMode::Panic => panic!("Interrupt {:?} already registered", int),
        }
    }
    set_ints.insert(int.int());
    unsafe {
        idt[int.int()].set_handler_fn(func);
        idt.load_unsafe();
    }
}

pub fn has_handler(int: InterruptId) -> bool {
    let set_ints = SET_INTS.lock();
    set_ints.contains(int.int())
}

/// Allocates handler somewhere in table. Return allocated interrupt
pub fn alloc_handler(func: HandlerFunc) -> Option<InterruptId> {
    let mut idt = IDT.lock();
    let mut set_ints = SET_INTS.lock();
    for i in 32..=255 {
        if set_ints.contains(i) {
            continue;
        }
        set_ints.insert(i);
        idt[i].set_handler_fn(func);
        unsafe {
            idt.load_unsafe();
        }
        return Some(InterruptId::new(i));
    }
    None
}

int_handler!(
    fpa_handler | stack_frame: InterruptStackFrame | {
        error!("EXCEPTION: SIMD FPA\n{:#?}", stack_frame);
    }
);

int_handler!(
    breakpoint_handler | stack_frame: InterruptStackFrame | {
        error!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
    }
);

int_handler!(
    divide_error_handler | stack_frame: InterruptStackFrame | {
        error!("EXCEPTION: DIVIDE ERROR\n{:#?}", stack_frame);
    }
);

int_handler!(
    overflow_handler | stack_frame: InterruptStackFrame | {
        error!("EXCEPTION: OVERFLOW\n{:#?}", stack_frame);
    }
);

int_handler!(
    bound_handler | stack_frame: InterruptStackFrame | {
        error!("EXCEPTION: BOUND RANGE EXCEEDED\n{:#?}", stack_frame);
    }
);

int_handler!(
    invalid_opcode | stack_frame: InterruptStackFrame | {
        error!("EXCEPTION: INVALID OPCODE\n{:#?}", stack_frame);
    }
);

int_handler!(
    device_not_available | stack_frame: InterruptStackFrame | {
        error!("EXCEPTION: FPU NOT AVAILABLE\n{:#?}", stack_frame);
    }
);

int_handler!(
    nmi_handler | stack_frame: InterruptStackFrame | {
        let status = nmi_status();
        panic!(
            "NMI: A = {:?}, B = {:?}\n{:#?}",
            status.0, status.1, stack_frame
        );
    }
);

extern "x86-interrupt" fn double_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) -> ! {
    panic!(
        "EXCEPTION: DOUBLE FAULT[{}]\n{:#?} => ",
        error_code, stack_frame
    );
}

extern "x86-interrupt" fn machine_check_handler(stack_frame: InterruptStackFrame) -> ! {
    panic!("HARDWARE FAULT\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn segment_not_present_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    error!("SEGMENT_NOT_PRESENT: {}\n{:#?}", error_code, stack_frame);
}

extern "x86-interrupt" fn general_protection_fault(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    error!(
        "EXCEPTION: GENERAL PROTECTION FAULT[{:?}]\n{:#?}",
        error_code, stack_frame
    );
}

extern "x86-interrupt" fn page_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: PageFaultErrorCode,
) {
    use x86_64::instructions::hlt;
    use x86_64::registers::control::Cr2;

    error!("EXCEPTION: PAGE FAULT");
    error!("Accessed Address: {:?}", Cr2::read());
    error!("Error Code: {:?}", error_code);
    error!("{:#?}", stack_frame);
    loop {
        hlt();
    }
}
