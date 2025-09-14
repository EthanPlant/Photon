use crate::{arch::interrupts::idt::IDT, interrupt_stack};

interrupt_stack!(divide_by_zero, |stack| {
    stack.dump();
    panic!("Divide by zero exception");
});

pub fn register_exceptions() {
    let mut idt = IDT.lock();
    unsafe {
        idt.set_handler(0, divide_by_zero);
    }
}
