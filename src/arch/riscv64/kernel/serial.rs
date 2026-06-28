use crate::synch::spinlock::SpinlockIrqSave;

use core::fmt;
use core::ptr::{read_volatile, write_volatile};

/// Base address of the NS16550 UART on the QEMU `virt` machine.
const NS16550_BASE: usize = 0x1000_0000;
/// Transmit holding register.
const UART_THR: *mut u8 = NS16550_BASE as *mut u8;
/// Line status register.
const UART_LSR: *const u8 = (NS16550_BASE + 5) as *const u8;
/// Transmit holding register empty.
const LSR_THRE: u8 = 1 << 5;

/// A serial interface to print messages, backed by the NS16550 UART.
pub(crate) struct ComPort;

impl ComPort {
	const fn new() -> Self {
		Self {}
	}

	/// Output a single byte, waiting until the transmit register is empty.
	fn put_byte(&self, b: u8) {
		unsafe {
			while read_volatile(UART_LSR) & LSR_THRE == 0 {
				core::hint::spin_loop();
			}
			write_volatile(UART_THR, b);
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
pub(crate) static COM1: SpinlockIrqSave<ComPort> = SpinlockIrqSave::new(ComPort::new());
