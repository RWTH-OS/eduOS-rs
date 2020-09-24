//! A wrapper around our serial console.

use crate::arch::serial;
use crate::synch::spinlock::*;
use core::fmt;

pub struct Console;

impl fmt::Write for Console {
	/// Output a string to each of our console outputs.
	fn write_str(&mut self, s: &str) -> fmt::Result {
		unsafe { serial::COM1.write_str(s) }
	}
}

pub static CONSOLE: SpinlockIrqSave<Console> = SpinlockIrqSave::new(Console);
