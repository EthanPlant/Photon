use crate::{drivers, logger};

mod gdt;
pub mod io;

#[derive(Debug, Clone, Copy)]
#[allow(dead_code)] // TODO: Replace this once we start supporting usermode
enum PrivilegeLevel {
    Kernel = 0,
    User = 3,
}

/// Entry point for `x86_64` architecture.
#[unsafe(no_mangle)]
pub extern "C" fn x86_64_main() -> ! {
    drivers::uart::init();
    logger::init();
    log::debug!("Serial logger initialized!");
    gdt::init();
    log::debug!("GDT... OK!");
    crate::kmain()
}

/// Halt the CPU indefinitely.
pub fn halt() -> ! {
    loop {
        // Safety: Halting the CPU is a safe operation in this context, as this largely a terminal state
        // and we want to stop all execution until the next interrupt.
        unsafe {
            core::arch::asm!("hlt");
        }
    }
}
