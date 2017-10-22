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
#![feature(core_intrinsics)]

#![no_std]

extern crate cpuio;
extern crate rlibc;
extern crate spin;
extern crate x86;
extern crate alloc;
extern crate alloc_kernel as allocator;
#[macro_use]
extern crate lazy_static;

// These need to be visible to the linker, so we need to export them.
pub use runtime_glue::*;
pub use logging::*;
pub use synch::semaphore::*;

#[macro_use]
mod macros;
#[macro_use]
mod logging;
mod runtime_glue;
pub mod consts;
pub mod arch;
pub mod console;
pub mod scheduler;
pub mod synch;

#[global_allocator]
static ALLOCATOR: allocator::Allocator = allocator::Allocator;

lazy_static! {
	static ref SEM: Semaphore = Semaphore::new(2);
}

extern "C" fn foo() {
	// simple demo, only 2 tasks are able to print at the same time
	SEM.acquire();

	for _i in 0..5 {
		println!("hello from task {}", scheduler::get_current_taskid());
		scheduler::reschedule();
	}

	SEM.release();
}

/// Rust entry point of the kernel
///
/// # Description
///
/// Boot loader calls this function to start the kernel
#[no_mangle]
pub extern "C" fn rust_main() {
	arch::init();
	scheduler::init();

	info!("Hello from eduOS-rs!");

	for _i in 0..4 {
		match scheduler::spawn(foo, scheduler::task::NORMAL_PRIO) {
			Ok(_id) => (),
			Err(why) => panic!("{:?}", why)
		}
	}

	match scheduler::spawn(foo, scheduler::task::REALTIME_PRIO) {
		Ok(_id) => (),
		Err(why) => panic!("{:?}", why)
	}

	loop {
		scheduler::reschedule();
		arch::processor::shutdown();
	}
}
