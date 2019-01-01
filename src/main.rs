#![feature(asm)]
#![no_std] // don't link the Rust standard library
#![cfg_attr(not(test), no_main)] // disable all Rust-level entry points
#![cfg_attr(test, allow(dead_code, unused_macros, unused_imports))]

#[macro_use]
extern crate eduos_rs;

use eduos_rs::arch;
use eduos_rs::mm;
use eduos_rs::scheduler;
use eduos_rs::scheduler::task::NORMAL_PRIORITY;
use eduos_rs::syscall;
use eduos_rs::syscall::{SYSNO_EXIT, SYSNO_MESSAGE};
use eduos_rs::{LogLevel,LOGGER};

extern "C" fn user_foo() -> ! {
	// try to call a kernel function => page fault
	//scheduler::do_exit();

	syscall!(SYSNO_MESSAGE);
	syscall!(SYSNO_EXIT);

	// we should never reach this point
	panic!("Syscall `exit` failed!");
}

extern "C" fn create_user_foo() {
	let cr3 = arch::x86_64::mm::paging::create_usr_pgd();

	unsafe {
		asm!("mov $0, %cr3" :: "r" (cr3) : "memory" : "volatile");
	}

	arch::x86_64::mm::paging::map_usr_entry(user_foo);

	debug!("jump to user land");
	arch::jump_to_user_land(user_foo);
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
	mm::init();
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
	arch::processor::shutdown();
}
