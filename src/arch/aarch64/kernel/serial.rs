use crate::synch::spinlock::Spinlock;

use core::fmt;
use core::ptr::{read_volatile, write_volatile};

/// Base address of the PL011 UART on the QEMU `virt` machine.
const PL011_BASE: usize = 0x0900_0000;
/// Data register (transmit/receive).
const UART_DR: *mut u8 = PL011_BASE as *mut u8;
/// Flag register.
const UART_FR: *const u32 = (PL011_BASE + 0x18) as *const u32;
/// Transmit FIFO full.
const FR_TXFF: u32 = 1 << 5;

/// A serial interface to print messages, backed by the PL011 UART.
pub(crate) struct ComPort;

impl ComPort {
	const fn new() -> Self {
		Self {}
	}

	/// Output a single byte, waiting until the transmit FIFO has room.
	fn put_byte(&self, b: u8) {
		unsafe {
			while read_volatile(UART_FR) & FR_TXFF != 0 {
				core::hint::spin_loop();
			}
			write_volatile(UART_DR, b);
		}
	}
}

impl fmt::Write for ComPort {
	/// Output a string to the serial interface. This allows using nice,
	/// high-level tools like Rust's `write!` macro.
	fn write_str(&mut self, s: &str) -> fmt::Result {
		for &b in s.as_bytes() {
			self.put_byte(b);
		}

		Ok(())
	}
}

/// Our primary serial port.
pub(crate) static COM1: Spinlock<ComPort> = Spinlock::new(ComPort::new());
