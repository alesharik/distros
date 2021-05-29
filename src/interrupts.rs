use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode, HandlerFunc};
use lazy_static::lazy_static;
use crate::{println, eprintln};
use crate::gdt;
use crate::kblog;
use spin::mutex::Mutex;
use crate::pic::nmi_status;
use fixedbitset::FixedBitSet;

pub const INT_LAPIC_TIMER: usize = 33;
pub const INT_LAPIC_ERROR: usize = 34;
pub const INT_LAPIC_SUPROUS: usize = 35;
pub const INT_IOAPIC_OFFSET: usize = 45;

lazy_static! {
    static ref IDT: Mutex<InterruptDescriptorTable> = Mutex::new({
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        idt.page_fault.set_handler_fn(page_fault_handler);
        idt.non_maskable_interrupt.set_handler_fn(nmi_handler);
        unsafe {
            idt.double_fault.set_handler_fn(double_fault_handler)
                .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
        }
        idt[INT_LAPIC_ERROR].set_handler_fn(lapic_error);
        idt[INT_LAPIC_SUPROUS].set_handler_fn(lapic_suprous);
        idt[INT_LAPIC_TIMER].set_handler_fn(lapic_timer);
        idt
    });
    static ref SET_INTS: Mutex<FixedBitSet> = Mutex::new(FixedBitSet::with_capacity(256));
}

pub fn init_idt() {
    let guard = IDT.lock();
    unsafe { guard.load_unsafe(); }
    kblog!("IDT", "IDT table loaded");
}

pub fn set_handler(int: usize, func: HandlerFunc) {
    let mut idt = IDT.lock();
    let mut set_ints = SET_INTS.lock();
    if set_ints.contains(int) {
        panic!("Interrupt {} already registered", int);
    }
    set_ints.insert(int);
    unsafe {
        idt[int].set_handler_fn(func);
        idt.load_unsafe();
    }
}

pub fn has_int_handler(int: usize) -> bool {
    let set_ints = SET_INTS.lock();
    set_ints.contains(int)
}

extern "x86-interrupt" fn breakpoint_handler(stack_frame: &mut InterruptStackFrame)  {
    println!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn nmi_handler(stack_frame: &mut InterruptStackFrame)  {
    let status = nmi_status();
    panic!("NMI: A = {:?}, B = {:?}\n{:#?}", status.0, status.1, stack_frame);
}

extern "x86-interrupt" fn lapic_error(stack_frame: &mut InterruptStackFrame)  {
    eprintln!("EXCEPTION: LAPIC ERROR\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn lapic_suprous(stack_frame: &mut InterruptStackFrame)  {
    eprintln!("EXCEPTION: LAPIC SUPROUS\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn lapic_timer(_stack_frame: &mut InterruptStackFrame)  {
    crate::pic::eoi();
}

extern "x86-interrupt" fn double_fault_handler(stack_frame: &mut InterruptStackFrame, _error_code: u64) -> ! {
    panic!("EXCEPTION: DOUBLE FAULT\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn page_fault_handler(stack_frame: &mut InterruptStackFrame, error_code: PageFaultErrorCode) {
    use x86_64::registers::control::Cr2;
    use x86_64::instructions::hlt;

    println!("EXCEPTION: PAGE FAULT");
    println!("Accessed Address: {:?}", Cr2::read());
    println!("Error Code: {:?}", error_code);
    println!("{:#?}", stack_frame);
    loop {
        hlt();
    }
}