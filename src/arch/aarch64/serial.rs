use core::fmt;
use core::ptr::write_bytes;
use hermit_sync::SpinMutex;

/// A COM serial port.
pub struct ComPort {
	/// base address of I/O registers.
	base_addr: u32,
}

impl ComPort {
	/// Create a new COM port with the specified base address.
	const fn new(base_addr: u32) -> Self {
		Self { base_addr }
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
				write_bytes(self.base_addr as *mut u8, b, 1);
			}
		}
		Ok(())
	}
}

/// Our primary serial port.
pub static COM1: SpinMutex<ComPort> = SpinMutex::new(ComPort::new(0x09000000));
