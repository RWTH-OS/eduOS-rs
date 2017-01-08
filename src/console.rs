//! A wrapper around both our VGA console and our serial console.

use core::fmt;
use spin::Mutex;
use arch::{vga, serial};

pub struct Console;

impl fmt::Write for Console {
    /// Output a string to each of our console outputs.
    fn write_str(&mut self, s: &str) -> fmt::Result {
        try!(vga::SCREEN.lock().write_str(s));
        serial::COM1.lock().write_str(s)
    }
}

pub static CONSOLE: Mutex<Console> = Mutex::new(Console);

