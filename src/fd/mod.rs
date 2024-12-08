pub(crate) mod stdio;

use crate::io;
use crate::scheduler::get_io_interface;

pub type FileDescriptor = i32;

pub const STDIN_FILENO: FileDescriptor = 0;
pub const STDOUT_FILENO: FileDescriptor = 1;
pub const STDERR_FILENO: FileDescriptor = 2;

#[allow(dead_code)]
pub(crate) trait IoInterface: Sync + Send + core::fmt::Debug {
	/// `read` attempts to read `len` bytes from the object references
	/// by the descriptor
	fn read(&self, _buf: &mut [u8]) -> io::Result<usize> {
		Err(io::Error::ENOSYS)
	}

	/// `write` attempts to write `len` bytes to the object references
	/// by the descriptor
	fn write(&self, _buf: &[u8]) -> io::Result<usize> {
		Err(io::Error::ENOSYS)
	}
}

pub(crate) fn write(fd: FileDescriptor, buf: &[u8]) -> io::Result<usize> {
	let obj = get_io_interface(fd)?;

	if buf.is_empty() {
		return Ok(0);
	}

	obj.write(buf)
}
