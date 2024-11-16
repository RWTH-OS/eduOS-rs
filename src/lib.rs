#![feature(allocator_api)]
#![feature(naked_functions)]
#![no_std]
#![no_main]

// These need to be visible to the linker, so we need to export them.
#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
pub(crate) use arch::processor::*;

#[macro_use]
pub mod macros;
#[macro_use]
pub(crate) mod logging;
pub(crate) mod arch;
pub mod console;

#[cfg(not(test))]
use core::panic::PanicInfo;

/// This function is called on panic.
#[cfg(not(test))]
#[panic_handler]
pub fn panic(info: &PanicInfo) -> ! {
	print!("[!!!PANIC!!!] ");

	if let Some(location) = info.location() {
		print!("{}:{}: ", location.file(), location.line());
	}

	if let Some(message) = info.message().as_str() {
		print!("{}", message);
	}

	print!("\n");

	shutdown(1);
}
