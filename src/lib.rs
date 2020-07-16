#![feature(llvm_asm, const_fn, lang_items)]
#![feature(allocator_api)]
#![feature(panic_info_message)]
#![feature(naked_functions)]
#![feature(abi_x86_interrupt)]
#![feature(specialization)]
#![no_std]

extern crate alloc;
extern crate spin;
#[cfg(target_arch = "x86_64")]
extern crate x86;
#[macro_use]
extern crate bitflags;
extern crate goblin;
extern crate num_traits;

// These need to be visible to the linker, so we need to export them.
#[cfg(target_arch = "x86_64")]
pub use arch::processor::*;
use core::panic::PanicInfo;
pub use logging::*;

#[macro_use]
pub mod macros;
#[macro_use]
pub mod logging;
pub mod arch;
pub mod collections;
pub mod console;
pub mod consts;
pub mod errno;
pub mod fs;
pub mod mm;
pub mod rlib;
pub mod scheduler;
pub mod synch;
pub mod syscall;

#[global_allocator]
static ALLOCATOR: &'static mm::allocator::Allocator = &mm::allocator::Allocator;

/// This function is called on panic.
#[cfg(not(test))]
#[panic_handler]
pub fn panic(info: &PanicInfo) -> ! {
	let tid = scheduler::get_current_taskid();

	print!("[!!!PANIC from task {}!!!] ", tid);

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
