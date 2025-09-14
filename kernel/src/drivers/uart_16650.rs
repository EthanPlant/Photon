use core::{
    fmt::{self, Write},
    marker::PhantomData,
};

use spin::{Mutex, Once};

use crate::arch;

const TRANSMIT_RECIEVE: u8 = 0;
const INTERRUPT_ENABLED: u8 = 1;
const BAUD_RATE_LSB: u8 = 0;
const BAUD_RATE_MSB: u8 = 1;
const FIFO_CONTROL: u8 = 2;
const LINE_CONTROL: u8 = 3;
const MODEM_CONTROL: u8 = 4;
const LINE_STATUS: u8 = 5;

const COM_1_ADDR: u16 = 0x3f8;

/// Global access to the COM1 serial port.
static COM_1: Once<Mutex<SerialPort<Initialized>>> = Once::new();

bitflags::bitflags! {
    /// Line Status Register
    #[derive(Debug, Copy, Clone)]
    struct LineStatus: u8 {
        /// The serial port has received data that can be read.
        /// This bit is cleared when the receive buffer is empty.
        const DATA_READY = 1;
        /// Overrun Error: Indicates that data was received but the previous data was not read,
        /// resulting in data loss.
        const OVERRUN_ERROR = 1 << 1;
        /// Parity Error: Indicates that the received data does not have the expected parity.
        const PARITY_ERROR = 1 << 2;
        /// Framing Error: Indicates that the received data does not have a valid stop bit.
        /// This can occur if the baud rate is incorrect or if there is noise on the line.
        const FRAMING_ERROR = 1 << 3;
        /// Break Interrupt: Indicates that the serial port has detected a break condition.
        /// A break condition occurs when the data line is held low for longer than a full
        /// word transmission time.
        const BREAK_INTERRUPT = 1 << 4;
        /// Transmitter Holding Register Empty: Indicates that the transmitter is ready to accept
        /// a new byte for transmission.
        const THR_EMPTY = 1 << 5;
        /// Transmitter Empty: Indicates that the transmitter is completely idle.
        /// This means that both the Transmitter Holding Register and the shift register are empty.
        const TRANSMITTER_EMPTY = 1 << 6;
        /// Error in Received FIFO: Indicates that at least one parity error, framing error,
        /// or break indication is present in the FIFO.
        const IMPENDING_ERROR = 1 << 7;
    }
}

/// Marker trait for the status of the serial port.
trait SerialStatus {}

/// The serial port hasn't been initialized and we cannot read or write to it.
struct Uninitialized;

/// The serial port has been initialized and is ready for reading and writing.
struct Initialized;

impl SerialStatus for Uninitialized {}
impl SerialStatus for Initialized {}

/// A struct representing a 16550 UART serial port.
/// The `S` generic parameter indicates the initialization status of the port.
///
/// - [`Uninitialized`]: The port has not been initialized and cannot be used for I/O.
/// - [`Initialized`]: The port has been initialized and is ready for I/O operations.
struct SerialPort<S: SerialStatus> {
    port: u16,
    status: PhantomData<S>,
}

impl<S: SerialStatus> SerialPort<S> {
    /// Write a value to a register of the serial port.
    ///
    /// # Safety
    /// This function is unsafe because it performs raw I/O operations.
    /// The caller must ensure that the port address and register are valid for writing.
    /// Also, writing to certain registers may have side effects.
    unsafe fn write_reg(&self, reg: u8, data: u8) {
        unsafe { arch::io::outb(self.port + reg as u16, data) };
    }

    /// Read a value from a register of the serial port.
    ///
    /// # Safety
    /// This function is unsafe because it performs raw I/O operations.
    /// The caller must ensure that the port address and register are valid for reading.
    /// Also, reading from certain registers may have side effects.
    unsafe fn read_reg(&self, reg: u8) -> u8 {
        unsafe { arch::io::inb(self.port + reg as u16) }
    }
}

impl SerialPort<Uninitialized> {
    /// Initialize the serial port.
    ///
    /// This function configures the serial port with standard settings:
    /// - Baud rate: 38400
    /// - Data bits: 8
    /// - Stop bits: 1
    /// - Parity: None
    /// - FIFO: Enabled, 14-byte threshold
    /// - Modem control: RTS and DTR enabled
    ///
    /// It also performs a self-test to ensure the port is functioning correctly.
    /// If the self-test fails, it returns `None`.
    /// If successful, it returns a `SerialPort<Initialized>`.
    ///
    /// # Safety
    /// This function is unsafe because it performs raw I/O operations.
    /// The caller must ensure that the port address is valid and that no other
    /// code is concurrently accessing the same port.
    unsafe fn init(&self) -> Option<SerialPort<Initialized>> {
        // Safety: The caller must ensure that no other code is accessing the same port.
        // and that the port address is valid.
        unsafe {
            self.write_reg(INTERRUPT_ENABLED, 0); // Disable all interrupts
            self.write_reg(LINE_CONTROL, 0x80); // Enable DLAB (set baud rate divisor)

            self.write_reg(BAUD_RATE_LSB, 0x03); // Set divisor to 3 (lo byte) 38400 baud
            self.write_reg(BAUD_RATE_MSB, 0x00); //                  (hi byte)
            self.write_reg(LINE_CONTROL, 0x03); // 8 bits, no parity, one stop bit
            self.write_reg(FIFO_CONTROL, 0xc7); // Enable FIFO, clear them, with 14-byte threshold
            self.write_reg(MODEM_CONTROL, 0x0b); // IRQs enabled, RTS/DSR set
        }

        if !self.self_test() {
            // Self-test failed. If this happens, the serial port is probably not
            // present or not functioning correctly and we cannot initialize it.
            return None;
        }

        // Safety: At this point we've initialized the port and know it is functional.
        // We can now enable interrupts and set the modem control register.
        unsafe { self.write_reg(MODEM_CONTROL, 0x0f) };

        Some(SerialPort::<Initialized> {
            port: self.port,
            status: PhantomData,
        })
    }

    fn self_test(&self) -> bool {
        // Safety: These are all valid operations for the serial port.
        unsafe {
            self.write_reg(MODEM_CONTROL, 0x1e);
            self.write_reg(TRANSMIT_RECIEVE, 0xae);
            self.read_reg(TRANSMIT_RECIEVE) == 0xae
        }
    }
}

impl SerialPort<Initialized> {
    /// Write a single byte to the serial port.
    ///
    /// This function waits until the transmitter holding register is empty
    /// before writing the byte to ensure that the byte is transmitted correctly.
    fn write_byte(&self, byte: u8) {
        self.wait_for_status(LineStatus::THR_EMPTY);

        // Safety: The serial port is initialized, the THR is empty, and the TRANSMIT_RECIEVE
        // register is valid for writing.
        unsafe {
            self.write_reg(TRANSMIT_RECIEVE, byte);
        }
    }

    fn get_line_status(&self) -> LineStatus {
        LineStatus::from_bits_truncate(unsafe { self.read_reg(LINE_STATUS) })
    }

    fn wait_for_status(&self, status: LineStatus) {
        while !self.get_line_status().contains(status) {
            core::hint::spin_loop();
        }
    }
}

impl fmt::Write for SerialPort<Initialized> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for byte in s.bytes() {
            self.write_byte(byte);
        }

        Ok(())
    }
}

/// Initialize the COM1 serial port and make it available for use.
/// This function should be called early in the boot process to ensure
/// that the serial port is ready for logging and debugging.
///
/// # Panics
/// This function will panic if the COM1 port fails to initialize or if it has already been initialized.
pub fn init() {
    // Safety: COM1 is a standard port address for the first serial port.
    let com_1 = unsafe {
        SerialPort::<Uninitialized> {
            port: COM_1_ADDR,
            status: PhantomData,
        }
        .init()
        .expect("COM1 failed to initialize")
    };

    COM_1.call_once(|| Mutex::new(com_1));
}

/// Print text to the serial port.
/// This macro works similarly to the standard `print!` macro,
/// but sends the output to the COM1 serial port instead.
/// If the serial port isn't initialized, this macro does nothing.
///
/// # Examples
/// ```
/// serial_print!("Hello, world!");
/// ```
#[macro_export]
macro_rules! serial_print {
    ($($arg:tt)*) => {
        ($crate::drivers::uart_16650::serial_print_internal(format_args!($($arg)*)))
    };
}

/// Print text to the serial port, followed by a newline.
/// This macro works similarly to the standard `println!` macro,
/// but sends the output to the COM1 serial port instead.
/// If the serial port isn't initialized, this macro does nothing.
///
/// # Examples
/// ```
/// serial_println!("Hello, world!");
/// ```
/// # Panics
/// This macro will panic if the serial port isn't initialized.
#[macro_export]
macro_rules! serial_println {
    () => {
        ($crate::serial_print!("\n"))
    };
    ($($arg:tt)*) => {
        ($crate::serial_print!("{}\n", format_args!($($arg)*)))
    };
}

#[doc(hidden)]
pub fn serial_print_internal(args: fmt::Arguments) {
    if let Some(com1) = COM_1.get() {
        com1.lock().write_fmt(args).unwrap();
    }
}
