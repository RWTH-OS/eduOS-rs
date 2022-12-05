#![feature(panic_info_message)]
#![feature(abi_x86_interrupt)]
#![no_std] // don't link the Rust standard library
#![cfg_attr(not(test), no_main)] // disable all Rust-level entry points
#![cfg_attr(test, allow(dead_code, unused_macros, unused_imports))]

#[macro_use]
extern crate eduos_rs;

#[allow(unused_imports)]
use core::fmt::Write;
use eduos_rs::arch;
use eduos_rs::arch::processor::shutdown;
use eduos_rs::scheduler;
use eduos_rs::scheduler::task::NORMAL_PRIORITY;
use eduos_rs::syscall;
use eduos_rs::syscall::{SYSNO_EXIT, SYSNO_WRITE};
use eduos_rs::{LogLevel, LOGGER};

extern "C" fn user_foo() {
	let str = b"Hello from user_foo!\n\0";

	unsafe {
		let _ = crate::arch::x86_64::kernel::serial::COM1.write_str("Hello from COM1!\n");
	}

	syscall!(SYSNO_WRITE, str.as_ptr() as u64, str.len());
	core::mem::forget(str);
	syscall!(SYSNO_EXIT);
}

extern "C" fn create_user_foo() {
	debug!("jump to user land");
	unsafe {
		arch::jump_to_user_land(user_foo);
	}
}

extern "C" fn foo() {
	println!("hello from task {}", scheduler::get_current_taskid());
}

/// This function is the entry point, since the linker looks for a function
/// named `_start` by default.
#[cfg(not(test))]
#[no_mangle] // don't mangle the name of this function
pub extern "C" fn main() -> ! {
	arch::init();
	scheduler::init();

	println!("Hello from eduOS-rs!");

	for _i in 0..2 {
		scheduler::spawn(foo, NORMAL_PRIORITY).unwrap();
	}
	scheduler::spawn(create_user_foo, NORMAL_PRIORITY).unwrap();

	// enable interrupts => enable preemptive multitasking
	arch::irq::irq_enable();

	scheduler::reschedule();

	println!("Shutdown system!");

	// shutdown system
	shutdown();
}
