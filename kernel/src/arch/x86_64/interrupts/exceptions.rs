use crate::{arch::interrupts::idt::IDT, interrupt_error, interrupt_stack};

interrupt_stack!(divide_by_zero, |stack| {
    stack.dump();
    panic!("Divide by zero exception");
});

interrupt_stack!(debug, |stack| {
    stack.dump();
    panic!("Debug exception")
});

interrupt_stack!(non_maskable_interrupt, |stack| {
    stack.dump();
    panic!("Non-maskable interrupt exception")
});

interrupt_stack!(breakpoint, |stack| {
    log::debug!("Breakpoint hit!");
    stack.dump();
});

interrupt_stack!(overflow, |stack| {
    stack.dump();
    panic!("Overflow exception")
});

interrupt_stack!(bound_range_exceeded, |stack| {
    stack.dump();
    panic!("Bound range exceeded exception")
});

interrupt_stack!(invalid_opcode, |stack| {
    stack.dump();
    panic!("Invalid opcode exception")
});

interrupt_stack!(device_not_available, |stack| {
    stack.dump();
    panic!("Device not available exception")
});

interrupt_error!(double_fault, |stack, error_code| {
    stack.dump();
    panic!("Double fault exception with error code: {}", error_code)
});

interrupt_error!(invalid_tss, |stack, error_code| {
    stack.dump();
    panic!("Invalid TSS exception with error code: {}", error_code)
});

interrupt_error!(segment_not_present, |stack, error_code| {
    stack.dump();
    panic!(
        "Segment not present exception with error code: {}",
        error_code
    )
});

interrupt_error!(stack_segment_fault, |stack, error_code| {
    stack.dump();
    panic!(
        "Stack segment fault exception with error code: {}",
        error_code
    )
});

interrupt_error!(general_protection_fault, |stack, error_code| {
    stack.dump();
    panic!(
        "General protection fault exception with error code: {}",
        error_code
    )
});

interrupt_error!(page_fault, |stack, error_code| {
    stack.dump();
    panic!("Page fault exception with error code: {}", error_code)
});

interrupt_stack!(x87_floating_point, |stack| {
    stack.dump();
    panic!("x87 floating point exception")
});

interrupt_error!(alignment_check, |stack, error_code| {
    stack.dump();
    panic!("Alignment check exception with error code: {}", error_code)
});

interrupt_stack!(machine_check, |stack| {
    stack.dump();
    panic!("Machine check exception")
});

interrupt_stack!(simd_floating_point, |stack| {
    stack.dump();
    panic!("SIMD floating point exception")
});

interrupt_stack!(virtualization, |stack| {
    stack.dump();
    panic!("Virtualization exception")
});

interrupt_error!(control_protection, |stack, error_code| {
    stack.dump();
    panic!(
        "Control protection exception with error code: {}",
        error_code
    )
});

interrupt_stack!(hypervisor_injection, |stack| {
    stack.dump();
    panic!("Hypervisor injection exception")
});

interrupt_error!(vmm_communication, |stack, error_code| {
    stack.dump();
    panic!(
        "VMM communication exception with error code: {}",
        error_code
    )
});

interrupt_error!(security_exception, |stack, error_code| {
    stack.dump();
    panic!("Security exception with error code: {}", error_code)
});

pub fn register_exceptions() {
    let mut idt = IDT.lock();
    unsafe {
        idt.set_handler(0, divide_by_zero);
        idt.set_handler(1, debug);
        idt.set_handler(2, non_maskable_interrupt);
        idt.set_handler(3, breakpoint);
        idt.set_handler(4, overflow);
        idt.set_handler(5, bound_range_exceeded);
        idt.set_handler(6, invalid_opcode);
        idt.set_handler(7, device_not_available);
        idt.set_handler(8, double_fault);
        idt.set_handler(10, invalid_tss);
        idt.set_handler(11, segment_not_present);
        idt.set_handler(12, stack_segment_fault);
        idt.set_handler(13, general_protection_fault);
        idt.set_handler(14, page_fault);
        idt.set_handler(16, x87_floating_point);
        idt.set_handler(17, alignment_check);
        idt.set_handler(18, machine_check);
        idt.set_handler(19, simd_floating_point);
        idt.set_handler(20, virtualization);
        idt.set_handler(21, control_protection);
        idt.set_handler(28, hypervisor_injection);
        idt.set_handler(29, vmm_communication);
        idt.set_handler(30, security_exception);
    }
}
