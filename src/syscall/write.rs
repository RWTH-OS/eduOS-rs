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

use alloc::string::String;

#[repr(C)]
pub struct IoVec {
	pub iov_base: *const u8,
	pub iov_len: usize
}

#[no_mangle]
pub unsafe extern "C" fn sys_writev(_fd: i32, ptr: *const IoVec, cnt: i32) -> isize
{
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
pub unsafe extern "C" fn sys_write(_fd: i32, s: *const u8, len: usize) -> isize
{
	let text = core::slice::from_raw_parts(s, len);

	print!("{}", String::from_utf8_lossy(text));

	len as isize
}
