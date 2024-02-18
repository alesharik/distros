#[macro_export]
macro_rules! int_handler {
    (pub noint $name:ident $body:expr) => {
        pub extern "x86-interrupt" fn $name(stack_frame: InterruptStackFrame) {
            x86_64::instructions::interrupts::without_interrupts(|| {
                $body(stack_frame);
            })
        }
    };
    (noint $name:ident $body:expr) => {
        extern "x86-interrupt" fn $name(stack_frame: InterruptStackFrame) {
            x86_64::instructions::interrupts::without_interrupts(|| {
                $body(stack_frame);
            })
        }
    };
    ($name:ident $body:expr) => {
        extern "x86-interrupt" fn $name(stack_frame: InterruptStackFrame) {
            $body(stack_frame)
        }
    };
}
