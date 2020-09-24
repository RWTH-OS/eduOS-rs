// Copyright (c) 2017-2018 Stefan Lankes, RWTH Aachen University
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

#[allow(unused_imports)]
use crate::scheduler::task::BOOT_STACK;

extern "C" {
	pub fn main();
}

#[cfg(not(test))]
#[no_mangle]
#[naked]
pub unsafe extern "C" fn _start() {
	// be sure that rsp is a valid stack pointer
	llvm_asm!("mov $0, %rsp" :: "r"(BOOT_STACK.top()) :: "volatile");

	main();

	loop {}
}
