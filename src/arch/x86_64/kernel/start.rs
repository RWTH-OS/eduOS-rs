// Copyright (c) 2017-2018 Stefan Lankes, RWTH Aachen University
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

use crate::consts::STACK_SIZE;
use crate::scheduler::task::BOOT_STACK;
use core::arch::asm;

extern "C" {
	pub fn main();
}

#[cfg(not(test))]
#[no_mangle]
pub unsafe extern "C" fn pre_main() -> ! {
	main();

	loop {}
}

#[cfg(not(test))]
#[no_mangle]
#[naked]
pub unsafe extern "C" fn _start() -> ! {
	asm!(
		// initialize stack pointer
		"lea rsp, [{stack}+{size}]",
		"call {pre_main}",
		stack = sym BOOT_STACK,
		size = const STACK_SIZE - 16,
		pre_main = sym pre_main,
		options(noreturn)
	);
}
