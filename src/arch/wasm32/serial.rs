use core::fmt;
use spin::Mutex;

extern {
	fn print(c: u16);
}

pub struct ComPort;

impl ComPort {
	const fn new() -> ComPort {
        ComPort { }
    }
}

impl fmt::Write for ComPort {
    /// Output a string to our COM port.  This allows using nice,
    /// high-level tools like Rust's `write!` macro.
    fn write_str(&mut self, s: &str) -> fmt::Result {
        // Output each byte of our string.
        for &b in s.as_bytes() {
            // Write our byte.
			unsafe { print(b as u16); }
        }
        Ok(())
    }
}

/// Our primary serial port.
pub static COM1: Mutex<ComPort> = Mutex::new(ComPort::new());
