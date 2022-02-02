// Copyright (c) 2017 Stefan Lankes, RWTH Aachen University
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

use crate::logging::*;
use crate::scheduler::*;
use core::arch::asm;

extern "C" fn invalid_syscall(sys_no: u64) -> ! {
	error!("Invalid syscall {}", sys_no);
	do_exit();
}

#[allow(unused_assignments)]
#[no_mangle]
#[naked]
pub unsafe extern "C" fn sys_invalid() {
	asm!("mov rdi, rax",
		"call {}",
		sym invalid_syscall,
		options(noreturn)
	);
}
