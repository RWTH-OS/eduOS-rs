use crate::synch::spinlock::Spinlock;
use core::fmt;
use semihosting::io::*;

/// A serial interface to print messages.
pub(crate) struct ComPort;

impl ComPort {
	const fn new() -> Self {
		Self {}
	}
}

impl fmt::Write for ComPort {
	/// Output a string to the serial interface. This allows using nice,
	/// high-level tools like Rust's `write!` macro.
	fn write_str(&mut self, s: &str) -> fmt::Result {
		stdout().unwrap().write(s.as_bytes()).unwrap();

		Ok(())
	}
}

/// Our primary serial port.
pub(crate) static COM1: Spinlock<ComPort> = Spinlock::new(ComPort::new());
