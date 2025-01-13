//! Implements basic functions to realize a simple in-memory file system

use crate::fd::OpenOption;
use crate::fs::SeekFrom;
use crate::io;
use crate::synch::spinlock::*;
use alloc::sync::Arc;
use alloc::vec::Vec;
use core::ops::{Deref, DerefMut};
use spinning_top::RwSpinlock;

#[derive(Debug)]
pub(crate) struct RomHandle {
	/// Position within the file
	pos: Spinlock<usize>,
	/// File content
	data: Arc<RwSpinlock<&'static [u8]>>,
}

impl RomHandle {
	pub fn new(slice: &'static [u8]) -> Self {
		RomHandle {
			pos: Spinlock::new(0),
			data: Arc::new(RwSpinlock::new(slice)),
		}
	}

	pub fn get_handle(&self, _opt: OpenOption) -> RomHandle {
		RomHandle {
			pos: Spinlock::new(0),
			data: self.data.clone(),
		}
	}

	pub fn read(&self, buf: &mut [u8]) -> io::Result<usize> {
		let vec = self.data.read();
		let mut pos_guard = self.pos.lock();
		let pos = *pos_guard;

		if pos >= vec.len() {
			return Ok(0);
		}

		let len;
		if vec.len() - pos < buf.len() {
			len = vec.len() - pos
		} else {
			len = buf.len()
		}

		buf[0..len].clone_from_slice(&vec[pos..pos + len]);
		*pos_guard = pos + len;

		Ok(len)
	}

	pub fn seek(&self, style: SeekFrom) -> io::Result<usize> {
		let mut pos_guard = self.pos.lock();

		match style {
			SeekFrom::Start(n) => {
				*pos_guard = n;
				Ok(n)
			}
			SeekFrom::End(n) => {
				let guard = self.data.read();
				let data = guard.len() as isize + n;
				if data >= 0 {
					*pos_guard = data as usize;
					Ok(data as usize)
				} else {
					Err(io::Error::EINVAL)
				}
			}
			SeekFrom::Current(n) => {
				let pos = *pos_guard as isize + n;
				if pos >= 0 {
					*pos_guard = pos as usize;
					Ok(pos as usize)
				} else {
					Err(io::Error::EINVAL)
				}
			}
		}
	}

	pub fn len(&self) -> usize {
		let guard = self.data.read();
		guard.len() as usize
	}
}

impl Clone for RomHandle {
	fn clone(&self) -> Self {
		RomHandle {
			pos: Spinlock::new(*self.pos.lock()),
			data: self.data.clone(),
		}
	}
}

#[derive(Debug)]
pub(crate) struct RamHandle {
	/// Is the file writeable?
	writeable: bool,
	/// Position within the file
	pos: Spinlock<usize>,
	/// File content
	data: Arc<RwSpinlock<Vec<u8>>>,
}

impl RamHandle {
	pub fn new(writeable: bool) -> Self {
		RamHandle {
			writeable: writeable,
			pos: Spinlock::new(0),
			data: Arc::new(RwSpinlock::new(Vec::new())),
		}
	}

	pub fn read(&self, buf: &mut [u8]) -> io::Result<usize> {
		let guard = self.data.read();
		let vec = guard.deref();
		let mut pos_guard = self.pos.lock();
		let pos = *pos_guard;

		if pos >= vec.len() {
			return Ok(0);
		}

		let len;
		if vec.len() - pos < buf.len() {
			len = vec.len() - pos
		} else {
			len = buf.len()
		}

		buf[0..len].clone_from_slice(&vec[pos..pos + len]);
		*pos_guard = pos + len;

		Ok(len)
	}

	pub fn write(&self, buf: &[u8]) -> io::Result<usize> {
		if self.writeable == false {
			return Err(io::Error::EBADF);
		}

		let mut guard = self.data.write();
		let vec = guard.deref_mut();
		let mut pos_guard = self.pos.lock();
		let pos = *pos_guard;

		if pos + buf.len() > vec.len() {
			vec.resize(pos + buf.len(), 0);
		}

		vec[pos..pos + buf.len()].clone_from_slice(buf);
		*pos_guard = pos + buf.len();

		Ok(buf.len())
	}

	pub fn seek(&self, style: SeekFrom) -> io::Result<usize> {
		let mut pos_guard = self.pos.lock();

		match style {
			SeekFrom::Start(n) => {
				*pos_guard = n as usize;
				Ok(n)
			}
			SeekFrom::End(n) => {
				let guard = self.data.read();
				let vec = guard.deref();
				let data = vec.len() as isize + n;
				if data >= 0 {
					*pos_guard = data as usize;
					Ok(data as usize)
				} else {
					Err(io::Error::EINVAL)
				}
			}
			SeekFrom::Current(n) => {
				let pos = *pos_guard as isize + n;
				if pos >= 0 {
					*pos_guard = pos as usize;
					Ok(pos as usize)
				} else {
					Err(io::Error::EINVAL)
				}
			}
		}
	}

	pub fn write_str(&self, s: &str) -> core::fmt::Result {
		if self.writeable == false {
			return Err(core::fmt::Error);
		}

		let mut guard = self.data.write();
		let vec = guard.deref_mut();
		let mut pos_guard = self.pos.lock();
		let pos = *pos_guard;

		if pos + s.len() > vec.len() {
			vec.resize(pos + s.len(), 0);
		}

		vec[pos..pos + s.len()].clone_from_slice(s.as_bytes());
		*pos_guard = pos + s.len();

		Ok(())
	}

	pub fn get_handle(&self, opt: OpenOption) -> RamHandle {
		RamHandle {
			writeable: opt.contains(OpenOption::O_RDWR),
			pos: Spinlock::new(0),
			data: self.data.clone(),
		}
	}

	pub fn len(&self) -> usize {
		let guard = self.data.read();
		let ref vec: &Vec<u8> = guard.deref();
		vec.len() as usize
	}
}

impl Clone for RamHandle {
	fn clone(&self) -> Self {
		RamHandle {
			writeable: self.writeable,
			pos: Spinlock::new(*self.pos.lock()),
			data: self.data.clone(),
		}
	}
}
