use crate::logging::*;
use alloc::string::String;

#[no_mangle]
pub(crate) extern "C" fn sys_write(s: *mut u8, len: usize) -> isize {
	debug!("enter syscall write");
	let str = unsafe { String::from_raw_parts(s, len, len) };
	print!("{}", str);
	core::mem::forget(str);

	len as isize
}
