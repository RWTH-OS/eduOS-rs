#![feature(const_mut_refs)]
#![feature(linked_list_cursors)]
#![feature(alloc_error_handler)]
#![feature(naked_functions)]
#![feature(abi_x86_interrupt)]
#![feature(specialization)]
#![allow(clippy::module_inception)]
#![allow(incomplete_features)]
#![allow(static_mut_refs)]
#![no_std]

extern crate alloc;
#[cfg(target_arch = "x86_64")]
extern crate x86;
#[macro_use]
extern crate bitflags;
extern crate num_traits;

// These need to be visible to the linker, so we need to export them.
use crate::arch::processor::shutdown;
use crate::consts::HEAP_SIZE;
use crate::synch::spinlock::RawSpinlock;
use core::panic::PanicInfo;
use core::ptr::addr_of;
pub use logging::*;
use talc::*;

#[macro_use]
pub mod macros;
#[macro_use]
pub mod logging;
pub mod arch;
pub mod collections;
pub mod console;
pub mod consts;
pub mod errno;
pub mod mm;
pub mod scheduler;
pub mod synch;
pub mod syscall;

static mut ARENA: [u8; HEAP_SIZE] = [0; HEAP_SIZE];

#[global_allocator]
static ALLOCATOR: Talck<RawSpinlock, ClaimOnOom> = Talc::new(unsafe {
	ClaimOnOom::new(Span::from_array(addr_of!(ARENA) as *mut [u8; HEAP_SIZE]))
})
.lock();

pub fn init() {
	crate::arch::init();
	crate::mm::init();
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
