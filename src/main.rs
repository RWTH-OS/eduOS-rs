#![feature(asm)]
#![no_std] // don't link the Rust standard library
#![cfg_attr(not(test), no_main)] // disable all Rust-level entry points
#![cfg_attr(test, allow(dead_code, unused_macros, unused_imports))]
#![feature(alloc)]

#[macro_use]
extern crate eduos_rs;
#[cfg(target_arch = "x86_64")]
extern crate x86;
extern crate alloc;

use eduos_rs::arch;
use eduos_rs::mm;
use eduos_rs::fs;
use eduos_rs::scheduler;
use eduos_rs::scheduler::task::NORMAL_PRIORITY;
use eduos_rs::{LogLevel,LOGGER};
use eduos_rs::arch::load_application;
use alloc::string::String;

extern "C" fn create_user_foo() {
	let path = String::from("/bin/demo");

	info!("Hello from loader");

	// load application
	if load_application(&path).is_err() {
		error!("Unable to load elf64 binary {}", path)
	}
}

extern "C" fn foo() {
	let tid = scheduler::get_current_taskid();

	println!("hello from task {}", tid);
}

/// This function is the entry point, since the linker looks for a function
/// named `_start` by default.
#[cfg(not(test))]
#[no_mangle] // don't mangle the name of this function
pub extern "C" fn main() -> ! {
	arch::init();
	mm::init();
	scheduler::init();
	fs::init();

	println!("Hello from eduOS-rs!");

	info!("Print file system:");
	fs::lsdir().unwrap();

	for _i in 0..2 {
		scheduler::spawn(foo, NORMAL_PRIORITY).unwrap();
	}
	scheduler::spawn(create_user_foo, NORMAL_PRIORITY).unwrap();

	// enable interrupts => enable preemptive multitasking
	arch::irq::irq_enable();

	scheduler::reschedule();

	println!("Shutdown system!");

	// shutdown system
	arch::processor::shutdown();
}
