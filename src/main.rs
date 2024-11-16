#![no_std] // don't link the Rust standard library
#![cfg_attr(not(test), no_main)] // disable all Rust-level entry points
#![cfg_attr(test, allow(dead_code, unused_macros, unused_imports))]

#[macro_use]
extern crate eduos_rs;

use eduos_rs::scheduler;
use eduos_rs::scheduler::task::{HIGH_PRIORITY, NORMAL_PRIORITY};
use eduos_rs::synch::mutex::Mutex;

static COUNTER: Mutex<u64> = Mutex::new(0);

extern "C" fn foo() {
	let mut guard = COUNTER.lock();

	for _i in 0..5 {
		*guard += 1;

		println!(
			"hello from task {}, counter {}",
			scheduler::get_current_taskid(),
			*guard
		);
		scheduler::reschedule();
	}
}

/// This function is the entry point, since the linker looks for a function
/// named `_start` by default.
#[cfg(not(test))]
#[no_mangle] // don't mangle the name of this function
pub extern "C" fn main() -> i32 {
	scheduler::init();

	println!("Hello from eduOS-rs!");

	for _i in 0..2 {
		scheduler::spawn(foo, NORMAL_PRIORITY).unwrap();
	}
	scheduler::spawn(foo, HIGH_PRIORITY).unwrap();

	scheduler::reschedule();

	println!("Shutdown system!");

	0
}
