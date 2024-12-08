use crate::fd::FileDescriptor;
use crate::logging::*;

pub(crate) extern "C" fn sys_write(fd: FileDescriptor, buf: *mut u8, len: usize) -> isize {
	debug!("Enter syscall write");
	let slice = unsafe { core::slice::from_raw_parts(buf, len) };
	crate::fd::write(fd, slice).map_or_else(
		|e| -num::ToPrimitive::to_isize(&e).unwrap(),
		|v| v.try_into().unwrap(),
	)
}
