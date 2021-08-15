use crate::gdt;
use crate::interrupts::pic::nmi_status;
use crate::interrupts::InterruptId;
use crate::kblog;
use crate::{eprintln, println};
use fixedbitset::FixedBitSet;
use lazy_static::lazy_static;
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
        unsafe {
            idt.double_fault
                .set_handler_fn(double_fault_handler)
                .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
        }
        idt[super::INT_LAPIC_ERROR.0].set_handler_fn(lapic_error);
        idt[super::INT_LAPIC_SUPROUS.0].set_handler_fn(lapic_suprous);
        idt[super::INT_LAPIC_TIMER.0].set_handler_fn(lapic_timer);
        idt
    });
    static ref SET_INTS: Mutex<FixedBitSet> = Mutex::new(FixedBitSet::with_capacity(256));
}

pub fn init_idt() {
    let guard = IDT.lock();
    unsafe {
        guard.load_unsafe();
    }
    kblog!("IDT", "IDT table loaded");
    // unsafe {
    // use x86_64::registers::control::{Cr4Flags, Cr4}; fixme enable, but it not works on my laptop
    // Cr4::update(|flags| {
    //     flags.set(Cr4Flags::MACHINE_CHECK_EXCEPTION, true);
    //     flags.set(Cr4Flags::USER_MODE_INSTRUCTION_PREVENTION, true);
    //     flags.set(Cr4Flags::TIMESTAMP_DISABLE, true);
    //     flags.set(Cr4Flags::SUPERVISOR_MODE_EXECUTION_PROTECTION, true);
    //     flags.set(Cr4Flags::SUPERVISOR_MODE_ACCESS_PREVENTION, true);
    // })
    // }
}

pub fn set_handler(int: InterruptId, func: HandlerFunc) {
    let mut idt = IDT.lock();
    let mut set_ints = SET_INTS.lock();
    if set_ints.contains(int.0) {
        panic!("Interrupt {} already registered", int.0);
    }
    set_ints.insert(int.0);
    unsafe {
        idt[int.0].set_handler_fn(func);
        idt.load_unsafe();
    }
}

pub fn has_int_handler(int: InterruptId) -> bool {
    let set_ints = SET_INTS.lock();
    set_ints.contains(int.0)
}

int_handler!(
    fpa_handler | stack_frame: InterruptStackFrame | {
        println!("EXCEPTION: SIMD FPA\n{:#?}", stack_frame);
    }
);

int_handler!(
    breakpoint_handler | stack_frame: InterruptStackFrame | {
        println!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
    }
);

int_handler!(
    divide_error_handler | stack_frame: InterruptStackFrame | {
        println!("EXCEPTION: DIVIDE ERROR\n{:#?}", stack_frame);
    }
);

int_handler!(
    overflow_handler | stack_frame: InterruptStackFrame | {
        println!("EXCEPTION: OVERFLOW\n{:#?}", stack_frame);
    }
);

int_handler!(
    bound_handler | stack_frame: InterruptStackFrame | {
        println!("EXCEPTION: BOUND RANGE EXCEEDED\n{:#?}", stack_frame);
    }
);

int_handler!(
    invalid_opcode | stack_frame: InterruptStackFrame | {
        println!("EXCEPTION: INVALID OPCODE\n{:#?}", stack_frame);
    }
);

int_handler!(
    device_not_available | stack_frame: InterruptStackFrame | {
        println!("EXCEPTION: FPU NOT AVAILABLE\n{:#?}", stack_frame);
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

int_handler!(
    lapic_error | stack_frame: InterruptStackFrame | {
        eprintln!("EXCEPTION: LAPIC ERROR\n{:#?}", stack_frame);
    }
);

int_handler!(
    lapic_suprous | stack_frame: InterruptStackFrame | {
        eprintln!("EXCEPTION: LAPIC SUPROUS\n{:#?}", stack_frame);
    }
);

int_handler!(noint lapic_timer |_stack_frame: InterruptStackFrame| {});

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

extern "x86-interrupt" fn general_protection_fault(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    println!(
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

    println!("EXCEPTION: PAGE FAULT");
    println!("Accessed Address: {:?}", Cr2::read());
    println!("Error Code: {:?}", error_code);
    println!("{:#?}", stack_frame);
    loop {
        hlt();
    }
}