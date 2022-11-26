// Copyright (c) 2018 Stefan Lankes, RWTH Aachen University
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

pub mod freelist;

use crate::arch;
use crate::arch::mm::get_memory_size;
use crate::logging::*;
#[cfg(not(test))]
use alloc::alloc::Layout;

pub fn init() {
	info!("Memory size {} MByte", get_memory_size() >> 20);

	arch::mm::init();
}

#[cfg(not(test))]
#[alloc_error_handler]
pub fn rust_oom(layout: Layout) -> ! {
	println!(
		"[!!!OOM!!!] Memory allocation of {} bytes failed",
		layout.size()
	);

	loop {}
}
