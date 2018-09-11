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

#![feature(asm, const_fn, lang_items)]
#![feature(panic_implementation)]
#![feature(panic_info_message)]
#![no_std]

extern crate spin;
#[cfg(target_arch = "x86_64")]
extern crate cpuio;
#[cfg(target_arch = "x86_64")]
extern crate x86;

// These need to be visible to the linker, so we need to export them.
pub use runtime_glue::*;
pub use logging::*;
#[cfg(target_arch = "x86_64")]
pub use arch::processor::*;

#[macro_use]
mod macros;
#[macro_use]
mod logging;
mod runtime_glue;
mod arch;
mod console;

#[no_mangle]
pub extern "C" fn rust_main() -> ! {
	println!("Hello world!");

	#[cfg(target_arch = "x86_64")]
	println!("CPU frequency {} MHz", arch::x86_64::get_cpufreq());

	// shutdown system
	shutdown();
}
