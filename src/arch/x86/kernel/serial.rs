use crate::synch::spinlock::SpinlockIrqSave;
use core::fmt;
use x86::io::*;

/// A COM serial port.
pub(crate) struct ComPort {
	/// COM ports are identified by the base address of their associated
	/// I/O registers.
	base_addr: u16,
}

impl ComPort {
	/// Create a new COM port with the specified base address.
	const fn new(base_addr: u16) -> Self {
		Self { base_addr }
	}

	pub fn write_bytes(&mut self, buf: &[u8]) {
		unsafe {
			// Output each byte of our string.
			for &b in buf {
				// Write our byte.
				outb(self.base_addr, b);
			}
		}
	}
}

impl fmt::Write for ComPort {
	/// Output a string to our COM port.  This allows using nice,
	/// high-level tools like Rust's `write!` macro.
	fn write_str(&mut self, s: &str) -> fmt::Result {
		unsafe {
			// Output each byte of our string.
			for &b in s.as_bytes() {
				// Write our byte.
				outb(self.base_addr, b);
			}
		}

		Ok(())
	}
}

/// Our primary serial port.
pub(crate) static COM1: SpinlockIrqSave<ComPort> = SpinlockIrqSave::new(ComPort::new(0x3F8));
