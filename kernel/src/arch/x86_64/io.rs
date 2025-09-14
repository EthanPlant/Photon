use core::arch::asm;

/// Write a byte to the specified I/O port.
///
/// # Safety
/// This function is unsafe because it performs raw I/O operations.
/// The caller must ensure that the port address is valid for writing.
/// It is also possible for writing to a port to have side effects.
pub unsafe fn outb(port: u16, value: u8) {
    unsafe { asm!("out dx, al", in("dx") port, in("al") value) };
}

/// Read a byte from the specified I/O port.
///
/// # Safety
/// This function is unsafe because it performs raw I/O operations.
/// The caller must ensure that the port address is valid for reading.
/// It is also possible for reading from a port to have side effects.
pub unsafe fn inb(port: u16) -> u8 {
    let value: u8;
    unsafe { asm!("in al, dx", in("dx") port, out("al") value) };
    value
}
