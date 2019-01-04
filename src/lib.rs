#![feature(asm, const_fn, lang_items)]
#![feature(alloc)]
#![feature(allocator_api)]
#![feature(panic_info_message)]
#![feature(integer_atomics)]
#![feature(compiler_builtins_lib)]
#![feature(naked_functions)]
#![feature(abi_x86_interrupt)]
#![feature(specialization)]
#![no_std]

#[cfg(target_arch = "x86_64")]
extern crate x86;
extern crate alloc;
#[macro_use]
extern crate bitflags;
extern crate num_traits;

// These need to be visible to the linker, so we need to export them.
pub use logging::*;
#[cfg(target_arch = "x86_64")]
pub use arch::processor::*;
use core::panic::PanicInfo;

#[macro_use]
pub mod macros;
#[macro_use]
pub mod logging;
pub mod consts;
pub mod arch;
pub mod console;
pub mod mm;
pub mod collections;
pub mod scheduler;
pub mod errno;
pub mod synch;
pub mod syscall;

#[global_allocator]
static ALLOCATOR: &'static mm::allocator::Allocator = &mm::allocator::Allocator;

/// This function is called on panic.
#[cfg(not(test))]
#[panic_handler]
pub fn panic(info: &PanicInfo) -> ! {
	print!("[!!!PANIC!!!] ");

	if let Some(location) = info.location() {
		print!("{}:{}: ", location.file(), location.line());
	}

	if let Some(message) = info.message() {
		print!("{}", message);
	}

	print!("\n");

	loop {
		halt();
	}
}
