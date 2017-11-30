// Copyright (c) 2017 Stefan Lankes, RWTH Aachen University
//
// MIT License
//
// Permission is hereby granted, free of charge, to any person obtaining
// a copy of this software and associated documentation files (the
// "Software"), to deal in the Software without restriction, including
// without limitation the rights to use, copy, modify, merge, publish,
// distribute, sublicense, and/or sell copies of the Software, and to
// permit persons to whom the Software is furnished to do so, subject to
// the following conditions:
//
// The above copyright notice and this permission notice shall be
// included in all copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND,
// EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF
// MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND
// NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE
// LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION
// OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION
// WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.

#![feature(asm)]
#![feature(const_fn)]
#![feature(lang_items)]
#![feature(repr_align)]
#![feature(attr_literals)]
#![feature(collections)]
#![feature(alloc, global_allocator, allocator_api, heap_api)]
#![feature(const_atomic_usize_new)]
#![feature(const_atomic_bool_new)]
#![feature(const_unsafe_cell_new)]
#![feature(const_shared_new)]
#![feature(abi_x86_interrupt)]
#![feature(shared)]
#![feature(hint_core_should_pause)]

#![no_std]

extern crate cpuio;
extern crate rlibc;
extern crate x86;
extern crate raw_cpuid;
extern crate alloc;
extern crate alloc_kernel as allocator;
extern crate multiboot;
#[macro_use]
extern crate lazy_static;

// These need to be visible to the linker, so we need to export them.
pub use runtime_glue::*;
pub use logging::*;
pub use synch::semaphore::*;
pub use synch::mutex::*;
pub use timer::*;
pub use syscall::*;

#[macro_use]
mod macros;
#[macro_use]
mod logging;
mod runtime_glue;
pub mod consts;
#[macro_use]
pub mod arch;
pub mod console;
pub mod scheduler;
pub mod synch;
pub mod timer;
pub mod syscall;

#[global_allocator]
static ALLOCATOR: allocator::Allocator = allocator::Allocator;

static SEM: Semaphore = Semaphore::new(2);

fn user_foo() -> ! {
	let str = b"Hello from user_foo!\n\0";

	//arch::x86_64::serial::COM1.write_str("Hello from user_foo!").unwrap();

	unsafe {
		syscall!(SYSNO_WRITE, str.as_ptr() as u64, str.len());
		syscall!(SYSNO_EXIT);
	}

	loop {
		arch::processor::halt();
	}
}

extern "C" fn create_user_foo() {
	debug!("jump to user land");
	arch::jump_to_user_land(user_foo);
}

extern "C" fn foo() {
	// simple demo, only 2 tasks are able to print at the same time
	SEM.acquire();

	for _i in 0..5 {
		println!("hello from task {}", scheduler::get_current_taskid());
		for _j in 0..100 {
			TIMER.msleep(20);
		}
	}

	SEM.release();
}

extern "C" fn initd() {
	info!("Hello from initd!");

	for _i in 0..4 {
		scheduler::spawn(foo, scheduler::task::NORMAL_PRIO);
	}

	scheduler::spawn(create_user_foo, scheduler::task::NORMAL_PRIO);
	scheduler::spawn(foo, scheduler::task::REALTIME_PRIO);
}

/// Rust entry point of the kernel
///
/// # Description
///
/// Boot loader calls this function to start the kernel
#[no_mangle]
pub extern "C" fn rust_main() -> ! {
	arch::init();
	scheduler::init();

	// enable interrupts => enable preemptive multitasking
	arch::irq::irq_enable();

	info!("Hello from eduOS-rs!");
	let freq = arch::processor::get_cpu_frequency();
	info!("Processor frequency: {} MHz", freq);

	scheduler::spawn(initd, scheduler::task::REALTIME_PRIO);

	loop {
		scheduler::reschedule();
		if scheduler::number_of_tasks() == 0 {
			arch::processor::shutdown();
		}
		arch::processor::halt();
	}
}
