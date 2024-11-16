#![feature(abi_x86_interrupt)]
#![feature(alloc_error_handler)]
#![feature(const_mut_refs)]
#![feature(const_refs_to_static)]
#![feature(const_trait_impl)]
#![feature(naked_functions)]
#![allow(clippy::module_inception)]
#![allow(static_mut_refs)]
#![no_std]

extern crate alloc;
#[cfg(target_arch = "x86_64")]
extern crate x86;

// These need to be visible to the linker, so we need to export them.
use crate::arch::processor::shutdown;
use crate::consts::HEAP_SIZE;
use core::panic::PanicInfo;
use core::ptr::addr_of;
use talc::*;

#[macro_use]
pub mod macros;
#[macro_use]
mod logging;
pub mod arch;
pub(crate) mod collections;
pub mod console;
pub(crate) mod consts;
pub mod errno;
pub(crate) mod mm;
pub mod scheduler;
pub mod synch;

static mut ARENA: [u8; HEAP_SIZE] = [0; HEAP_SIZE];

#[global_allocator]
static ALLOCATOR: Talck<spin::Mutex<()>, ClaimOnOom> = Talc::new(unsafe {
	ClaimOnOom::new(Span::from_array(addr_of!(ARENA) as *mut [u8; HEAP_SIZE]))
})
.lock();

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
