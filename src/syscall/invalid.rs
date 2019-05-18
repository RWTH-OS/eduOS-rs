// Copyright (c) 2017 Stefan Lankes, RWTH Aachen University
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

use logging::*;
use scheduler::*;

#[no_mangle]
#[naked]
pub unsafe extern "C" fn sys_invalid()
{
	let mut rax: i64 = 0;

	asm!("push %rax; pop $0" : "=r"(rax));

	error!("Invalid syscall {}", rax);
	do_exit();
}
