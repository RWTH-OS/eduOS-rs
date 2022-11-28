// Copyright (c) 2017 Stefan Lankes, RWTH Aachen University
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

use alloc::string::String;

#[repr(C)]
pub struct IoVec {
	pub iov_base: *const u8,
	pub iov_len: usize,
}

#[no_mangle]
pub unsafe extern "C" fn sys_writev(_fd: i32, ptr: *const IoVec, cnt: i32) -> isize {
	let mut len: isize = 0;
	let iovec = core::slice::from_raw_parts(ptr, cnt as usize);

	for i in iovec {
		let s = core::slice::from_raw_parts(i.iov_base, i.iov_len);

		len += i.iov_len as isize;
		print!("{}", String::from_utf8_lossy(s));
	}

	len
}

#[no_mangle]
pub unsafe extern "C" fn sys_write(_fd: i32, s: *mut u8, len: usize) -> isize {
	let str = unsafe { String::from_raw_parts(s, len, len) };
	print!("{}", str);
	core::mem::forget(str);

	len as isize
}
