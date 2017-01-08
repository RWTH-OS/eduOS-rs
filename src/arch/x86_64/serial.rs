//! Basic serial port driver.
//!
//! As usual, inspired by http://wiki.osdev.org/Serial_Ports

use core::fmt;
use spin::Mutex;
use cpuio;
use self::Register::*;

/// Each COM port has 8 I/O registers associated with it, some of which are
/// dual use.
#[allow(dead_code)]
#[repr(C, u8)]
enum Register {
    /// Either a data register or the low byte of our baud divisor, depending
    /// on the high bit of LineControl.
    DataOrBaudLsb = 0,
    /// Either a interrupt flags register or the high byte of our baud
    /// divisor.
    InterruptEnableOrBaudMsb = 1,
    InterruptIdentAndFifo = 2,
    /// When the high bit of LineControl is set, the first two registers
    /// switch over to their "baud mode".
    LineControl = 3,
    ModemControl = 4,
    LineStatus = 5,
    ModemStatus = 6,
    Scratch = 7
}

/// A COM serial port.
pub struct ComPort {
    /// COM ports are identified by the base address of their associated
    /// I/O registers.
    base_addr: u16,
    /// Has this port been initialized yet?
    initialized: bool,
}

impl ComPort {
    /// Create a new COM port with the specified base address.  Note that
    /// this does not actually finish initializing the serial port, because
    /// this is `const` function that may be computed at compile time.
    /// Initialization is finished by `lazy_initialize`, which should be
    /// called by all safe, public functions in this API.
    const unsafe fn new(base_addr: u16) -> ComPort {
        ComPort { base_addr: base_addr, initialized: false }
    }

    /// Finish the runtime-only setup needed by this port.
    unsafe fn lazy_initialize(&mut self) {
        if self.initialized == true { return; }
        self.initialized = true;

        // Disable interrupts.
        self.port(InterruptEnableOrBaudMsb).write(0x00);

        // Set baud and 8N1 mode.
        self.set_baud_divisor(2); // 115,200 / 2
        self.port(LineControl).write(0x03);

        // Enable FIFOs with 14-byte threshhold.
        self.port(InterruptIdentAndFifo).write(0xC7);

        // Configure modem: RTS/DSR and IRQs on.  But if we actually want
        // to get IRQs, I think we also need to set up
        // InterruptEnableOrBaudMsb.
        self.port(ModemControl).write(0x0B);
    }

    /// Get an cpuio::Port object for one of our associated ports.  This is
    /// marked as `unsafe` because the returned port can potentially be
    /// used to mess with processor interrupts and otherwise violate
    /// fundamental abstractions about how Rust code works.
    unsafe fn port(&mut self, register: Register) -> cpuio::Port<u8> {
        cpuio::Port::new(self.base_addr + (register as u8 as u16))
    }

    /// Set the baud rate as a divisor of 115,200.
    fn set_baud_divisor(&mut self, divisor: u16) {
        unsafe {
            self.lazy_initialize();

            // Switch ports DataOrBaudLsb and InterruptEnableOrBaudMsb to
            // their baud mode by setting the high bit of LineControl.
            let saved_line_control = self.port(LineControl).read();
            self.port(LineControl).write(0x80 | saved_line_control);

            // Set baud divisor.
            self.port(DataOrBaudLsb).write(divisor as u8);
            self.port(InterruptEnableOrBaudMsb).write((divisor >> 8) as u8);

            // Restore previous port modes.
            self.port(LineControl).write(saved_line_control);
        }
    }

    /// Can we safely transmit data on this serial port right now, or will
    /// we block?
    fn can_transmit(&mut self) -> bool {
        unsafe {
            self.lazy_initialize();
            // TODO: Check to see what the meaning of this bit is. OSDev
            // calls it "is_transmit_empty", so maybe we actually want a
            // different bit.
            (self.port(LineStatus).read() & 0x20) != 0
        }
    }
}

impl fmt::Write for ComPort {
    /// Output a string to our COM port.  This allows using nice,
    /// high-level tools like Rust's `write!` macro.
    fn write_str(&mut self, s: &str) -> fmt::Result {
        unsafe {
            self.lazy_initialize();

            // Output each byte of our string.
            for &b in s.as_bytes() {
                // Loop until the port's available.
                while !self.can_transmit() {}

                // Write our byte.
                self.port(DataOrBaudLsb).write(b);
            }
        }
        Ok(())
    }
}

/// Our primary serial port.
pub static COM1: Mutex<ComPort> = Mutex::new(unsafe {
    ComPort::new(0x03F8)
});
