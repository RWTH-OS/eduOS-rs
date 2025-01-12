pub(crate) mod stdio;

use crate::io;
use crate::scheduler::get_io_interface;

pub type FileDescriptor = i32;

pub const STDIN_FILENO: FileDescriptor = 0;
pub const STDOUT_FILENO: FileDescriptor = 1;
pub const STDERR_FILENO: FileDescriptor = 2;

/// Enumeration of possible methods to seek within an I/O object.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum SeekFrom {
	/// Set the offset to the provided number of bytes.
	Start(usize),
	/// Set the offset to the size of this object plus the specified number of bytes.
	///
	/// It is possible to seek beyond the end of an object, but it's an error to
	/// seek before byte 0.
	End(isize),
	/// Set the offset to the current position plus the specified number of bytes.
	///
	/// It is possible to seek beyond the end of an object, but it's an error to
	/// seek before byte 0.
	Current(isize),
}

/// Describes information about a file.
pub struct FileStatus {
	/// Size of the file
	pub file_size: usize,
}

#[allow(dead_code)]
pub trait IoInterface: Sync + Send + core::fmt::Debug {
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

	fn seek(&self, _offset: SeekFrom) -> io::Result<usize> {
		Err(io::Error::ENOSYS)
	}

	fn fstat(&self) -> io::Result<FileStatus> {
		Err(io::Error::ENOSYS)
	}
}

bitflags! {
	/// Options for opening files
	#[derive(Debug, Copy, Clone)]
	pub struct OpenOption: i32 {
		const O_RDONLY = 0o0000;
		const O_WRONLY = 0o0001;
		const O_RDWR = 0o0002;
		const O_CREAT = 0o0100;
		const O_EXCL = 0o0200;
		const O_TRUNC = 0o1000;
		const O_APPEND = 0o2000;
		const O_DIRECT = 0o40000;
		const O_DIRECTORY = 0o200_000;
	}
}

pub(crate) fn read(fd: FileDescriptor, buf: &mut [u8]) -> io::Result<usize> {
	let obj = get_io_interface(fd)?;

	if buf.is_empty() {
		return Ok(0);
	}

	obj.read(buf)
}

pub(crate) fn write(fd: FileDescriptor, buf: &[u8]) -> io::Result<usize> {
	let obj = get_io_interface(fd)?;

	if buf.is_empty() {
		return Ok(0);
	}

	obj.write(buf)
}

pub(crate) fn fstat(fd: FileDescriptor) -> io::Result<FileStatus> {
	get_io_interface(fd)?.fstat()
}
