use core::arch::asm;

pub mod exceptions;
mod handler;
pub mod idt;

/// Wrapper around the `cli` instruction to disable interrupts
pub fn disable_interrupts() {
    // Safety: It is always safe to call `cli`
    unsafe {
        asm!("cli", options(nomem, nostack));
    }
}

/// Wrapper around the `sti` instruction to enable interrupts
pub fn enable_interrupts() {
    // Safety: It is always safe to call `sti`
    unsafe {
        asm!("sti", options(nomem, nostack));
    }
}
