//! A wrapper around our serial console.

#[cfg(not(all(feature = "vga", any(target_arch = "x86_64", target_arch = "x86"))))]
use crate::arch::serial;
#[cfg(all(feature = "vga", any(target_arch = "x86_64", target_arch = "x86")))]
use crate::arch::vga;
use core::fmt;

pub struct Console;

impl fmt::Write for Console {
	/// Output a string to each of our console outputs.
	fn write_str(&mut self, s: &str) -> fmt::Result {
		cfg_if::cfg_if! {
			if #[cfg(all(feature = "vga", any(target_arch = "x86_64", target_arch = "x86")))] {
				vga::VGA_SCREEN.lock().write_str(s)
			} else {
				serial::COM1.lock().write_str(s)
			}
		}
	}
}
