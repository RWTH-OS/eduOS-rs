#[cfg(not(feature = "vga"))]
use crate::arch::serial;
#[cfg(feature = "vga")]
use crate::arch::vga;
use crate::fd::IoInterface;
use crate::io;

#[derive(Debug)]
pub(crate) struct GenericStdin;

impl IoInterface for GenericStdin {}

impl GenericStdin {
	pub const fn new() -> Self {
		Self {}
	}
}

#[derive(Debug)]
pub(crate) struct GenericStdout;

impl IoInterface for GenericStdout {
	fn write(&self, buf: &[u8]) -> io::Result<usize> {
		cfg_if::cfg_if! {
			if #[cfg(feature = "vga")] {
				vga::VGA_SCREEN.lock().write_bytes(buf);
			} else {
				serial::COM1.lock().write_bytes(buf);
			}
		}

		Ok(buf.len())
	}
}

impl GenericStdout {
	pub const fn new() -> Self {
		Self {}
	}
}

#[derive(Debug)]
pub(crate) struct GenericStderr;

impl IoInterface for GenericStderr {
	fn write(&self, buf: &[u8]) -> io::Result<usize> {
		cfg_if::cfg_if! {
			if #[cfg(feature = "vga")] {
				vga::VGA_SCREEN.lock().write_bytes(buf);
			} else {
				serial::COM1.lock().write_bytes(buf);
			}
		}

		Ok(buf.len())
	}
}

impl GenericStderr {
	pub const fn new() -> Self {
		Self {}
	}
}
