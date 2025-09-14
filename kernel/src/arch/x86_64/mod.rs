use crate::{drivers, kmain, serial_println};

pub mod io;

/// Entry point for x86_64 architecture.
#[unsafe(no_mangle)]
pub extern "C" fn x86_64_main() -> ! {
    drivers::uart::init();
    serial_println!("Hello, world!");
    kmain()
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
