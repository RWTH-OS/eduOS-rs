#![feature(alloc_error_handler)]
#![feature(const_trait_impl)]
#![allow(clippy::module_inception)]
#![allow(static_mut_refs)]
#![no_std]

extern crate alloc;
extern crate spin;
#[cfg(target_arch = "x86_64")]
extern crate x86;

// These need to be visible to the linker, so we need to export them.
use crate::arch::processor::shutdown;
use crate::consts::HEAP_SIZE;
use crate::mm::buddy::LockedHeap;
use core::panic::PanicInfo;

#[macro_use]
pub mod macros;
#[macro_use]
mod logging;
pub mod arch;
pub mod console;
pub mod consts;
pub mod errno;
pub mod mm;
pub mod scheduler;

#[global_allocator]
static ALLOCATOR: LockedHeap<32> = LockedHeap::<32>::new();

#[repr(align(256))]
struct Arena([u8; HEAP_SIZE]);

impl Arena {
	pub const fn new() -> Self {
		Self([0; HEAP_SIZE])
	}
}

static mut ARENA: Arena = Arena::new();

pub fn init() {
	unsafe {
		crate::ALLOCATOR.init(ARENA.0.as_mut_ptr(), HEAP_SIZE);
	}
	crate::scheduler::init();
}

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
