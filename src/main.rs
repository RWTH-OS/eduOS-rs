#![feature(abi_x86_interrupt)]
#![no_std] // don't link the Rust standard library
#![cfg_attr(not(test), no_main)] // disable all Rust-level entry points
#![cfg_attr(test, allow(dead_code, unused_macros, unused_imports))]

#[macro_use]
extern crate eduos_rs;

use eduos_rs::arch;
use eduos_rs::scheduler;

extern "C" fn foo() {
	for _i in 0..5 {
		println!("hello from task {}", scheduler::get_current_taskid());
		scheduler::reschedule();
	}
}

/// This function is the entry point, since the linker looks for a function
/// named `_start` by default.
#[cfg(not(test))]
#[no_mangle] // don't mangle the name of this function
pub extern "C" fn main() -> i32 {
	arch::init();
	scheduler::init();

	println!("Hello from eduOS-rs!");

	for _i in 0..2 {
		scheduler::spawn(foo);
	}

	scheduler::reschedule();

	println!("Shutdown system!");

	0
}
