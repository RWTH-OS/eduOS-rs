#![feature(panic_info_message)]
#![feature(abi_x86_interrupt)]
#![no_std] // don't link the Rust standard library
#![cfg_attr(not(test), no_main)] // disable all Rust-level entry points
#![cfg_attr(test, allow(dead_code, unused_macros, unused_imports))]

#[macro_use]
extern crate eduos_rs;

use eduos_rs::arch;
use eduos_rs::arch::processor::shutdown;
use eduos_rs::scheduler;
use eduos_rs::scheduler::task::{HIGH_PRIORITY, NORMAL_PRIORITY};
use eduos_rs::synch::mutex::Mutex;

static mut COUNTER: Option<Mutex<u64>> = None;

extern "C" fn foo() {
	let mut guard = unsafe {
		match COUNTER {
			Some(ref mut c) => c.lock(),
			None => {
				panic!("Mutex isn't initialized");
			}
		}
	};

	for _i in 0..5 {
		*guard += 1;

		println!(
			"hello from task {}, counter {}",
			scheduler::get_current_taskid(),
			0
		); //*guard);
		scheduler::reschedule();
	}
}

/// This function is the entry point, since the linker looks for a function
/// named `_start` by default.
#[cfg(not(test))]
#[no_mangle] // don't mangle the name of this function
pub extern "C" fn main() -> ! {
	scheduler::init();

	println!("Hello from eduOS-rs!");

	unsafe {
		COUNTER = Some(Mutex::new(0));
	}

	for _i in 0..2 {
		scheduler::spawn(foo, NORMAL_PRIORITY).unwrap();
	}
	scheduler::spawn(foo, HIGH_PRIORITY).unwrap();

	scheduler::reschedule();

	println!("Shutdown system!");

	// shutdown system
	shutdown();
}
