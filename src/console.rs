//! A wrapper around our serial console.

use core::fmt;
use spin::Mutex;
use arch::serial;

pub struct Console;

impl fmt::Write for Console {
	/// Output a string to each of our console outputs.
	fn write_str(&mut self, s: &str) -> fmt::Result {
		serial::COM1.lock().write_str(s)
	}
}

pub static CONSOLE: Mutex<Console> = Mutex::new(Console);
