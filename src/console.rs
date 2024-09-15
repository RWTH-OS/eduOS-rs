//! A wrapper around our serial console.

#[cfg(not(all(target_arch = "x86", feature = "vga")))]
use crate::arch::serial;
#[cfg(all(target_arch = "x86", feature = "vga"))]
use crate::arch::vga;
use core::fmt;

pub struct Console;

impl fmt::Write for Console {
	/// Output a string to each of our console outputs.
	fn write_str(&mut self, s: &str) -> fmt::Result {
		cfg_if::cfg_if! {
			if #[cfg(all(target_arch = "x86", feature = "vga"))] {
				vga::VGA_SCREEN.lock().write_str(s)
			} else {
				serial::COM1.lock().write_str(s)
			}
		}
	}
}
