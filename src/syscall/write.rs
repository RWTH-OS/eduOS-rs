// Copyright (c) 2017 Stefan Lankes, RWTH Aachen University
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

use crate::logging::*;
use alloc::string::String;

#[no_mangle]
pub extern "C" fn sys_write(s: *mut u8, len: usize) -> isize {
	debug!("enter syscall write");
	let str = unsafe { String::from_raw_parts(s, len, len) };
	print!("{}", str);
	core::mem::forget(str);

	len as isize
}
