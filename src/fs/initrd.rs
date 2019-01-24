// Copyright (c) 2019 Stefan Lankes, RWTH Aachen University
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

//! Implements basic functions to realize a simple in-memory file system

use errno::*;
use fs::{OpenOptions, SeekFrom};
use alloc::vec::Vec;
use alloc::sync::Arc;
use core::ops::{Deref,DerefMut};
use core::slice;
use spin::RwLock;
use synch::spinlock::*;

#[derive(Debug)]
pub struct RomHandle {
	/// Position within the file
	pos: Spinlock<usize>,
	/// File content
	data: Arc<RwLock<&'static [u8]>>
}

impl RomHandle {
	pub fn new(addr: *const u8, len: usize) -> Self {
		RomHandle {
			pos: Spinlock::new(0),
			data: Arc::new(RwLock::new(unsafe { slice::from_raw_parts(addr, len) } ))
		}
	}

	pub fn get_handle(&self, _opt: OpenOptions) -> RomHandle {
		RomHandle {
			pos: Spinlock::new(0),
			data: self.data.clone()
		}
	}

	pub fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
		let vec = self.data.read();
		let mut pos_guard = self.pos.lock();
		let pos = *pos_guard;

		if pos >= vec.len() {
			return Ok(0)
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

	pub fn seek(&mut self, style: SeekFrom) -> Result<u64> {
		let mut pos_guard = self.pos.lock();

		match style {
			SeekFrom::Start(n) => {
				*pos_guard = n as usize;
				Ok(n)
			}
			SeekFrom::End(n) => {
				let guard = self.data.read();
				let data = guard.len() as i64 + n;
				if data >= 0 {
					*pos_guard = data as usize;
					Ok(data as u64)
				} else {
					Err(Error::InvalidArgument)
				}
			}
			SeekFrom::Current(n) => {
				let pos = *pos_guard as i64 + n;
				if pos >= 0 {
					*pos_guard = pos as usize;
					Ok(pos as u64)
				} else {
					Err(Error::InvalidArgument)
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
			data: self.data.clone()
		}
	}
}

#[derive(Debug)]
pub struct RamHandle {
	/// Is the file writeable?
	writeable: bool,
	/// Position within the file
	pos: Spinlock<usize>,
	/// File content
	data: Arc<RwLock<Vec<u8>>>
}

impl RamHandle {
	pub fn new(writeable: bool) -> Self {
		RamHandle {
			writeable: writeable,
			pos: Spinlock::new(0),
			data: Arc::new(RwLock::new(Vec::new()))
		}
	}

	pub fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
		let guard = self.data.read();
		let ref vec: &Vec<u8> = guard.deref();
		let mut pos_guard = self.pos.lock();
		let pos = *pos_guard;

		if pos >= vec.len() {
			return Ok(0)
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

	pub fn write(&mut self, buf: &[u8]) -> Result<usize> {
		if self.writeable == false {
			return Err(Error::BadFsPermission);
		}

		let mut guard = self.data.write();
		let ref mut vec: &mut Vec<u8> = guard.deref_mut();
		let mut pos_guard = self.pos.lock();
		let pos = *pos_guard;

		if pos + buf.len() > vec.len() {
			vec.resize(pos + buf.len(), 0);
		}

		vec[pos..pos + buf.len()].clone_from_slice(buf);
		*pos_guard = pos + buf.len();

		Ok(buf.len())
	}

	pub fn seek(&mut self, style: SeekFrom) -> Result<u64> {
		let mut pos_guard = self.pos.lock();

		match style {
			SeekFrom::Start(n) => {
				*pos_guard = n as usize;
				Ok(n)
			}
			SeekFrom::End(n) => {
				let guard = self.data.read();
				let ref vec: &Vec<u8> = guard.deref();
				let data = vec.len() as i64 + n;
				if data >= 0 {
					*pos_guard = data as usize;
					Ok(data as u64)
				} else {
					Err(Error::InvalidArgument)
				}
			}
			SeekFrom::Current(n) => {
				let pos = *pos_guard as i64 + n;
				if pos >= 0 {
					*pos_guard = pos as usize;
					Ok(pos as u64)
				} else {
					Err(Error::InvalidArgument)
				}
			}
		}
	}

	pub fn write_str(&mut self, s: &str) -> core::fmt::Result {
		if self.writeable == false {
			return Err(core::fmt::Error);
		}

		let mut guard = self.data.write();
		let ref mut vec: &mut Vec<u8> = guard.deref_mut();
		let mut pos_guard = self.pos.lock();
		let pos = *pos_guard;

		if pos + s.len() > vec.len() {
			vec.resize(pos + s.len(), 0);
		}

		vec[pos..pos + s.len()].clone_from_slice(s.as_bytes());
		*pos_guard = pos + s.len();

		Ok(())
	}

	pub fn get_handle(&self, opt: OpenOptions) -> RamHandle {
		RamHandle {
			writeable: opt.contains(OpenOptions::READWRITE),
			pos: Spinlock::new(0),
			data: self.data.clone()
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
			data: self.data.clone()
		}
	}
}
