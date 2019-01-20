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

use logging::*;
use errno::*;
use fs::{NodeKind,VfsNode, Vfs, OpenOptions, FileHandle};
use alloc::boxed::Box;
use alloc::vec::Vec;
use alloc::string::String;
use alloc::collections::BTreeMap;
use alloc::sync::Arc;
use core::ops::{Deref,DerefMut};
use core::fmt;
use spin::RwLock;
use synch::spinlock::*;

#[derive(Debug)]
pub struct MemoryFsDirectory {
	/// Director name
	name: String,
	/// Queue of VfsNode
	children: BTreeMap<String, Box<VfsNode>>
}

impl MemoryFsDirectory {
	pub fn new(name: String) -> Self {
		MemoryFsDirectory {
			name: name,
			children: BTreeMap::new()
		}
	}
}

impl VfsNode for MemoryFsDirectory {
	fn get_name(&self) -> String {
		self.name.clone()
	}

	fn get_kind(&self) -> NodeKind {
		NodeKind::Directory
	}

	fn traverse_mkdir(&mut self, components: &mut Vec<&str>) -> Result<()> {
    	if let Some(component) = components.pop() {
			let directory = &mut self.children.entry(String::from(component))
                            .or_insert(Box::new(MemoryFsDirectory::new(String::from(component))));
        	if directory.get_kind() != NodeKind::Directory {
            	return Err(Error::BadFsKind);
        	}

        	directory.traverse_mkdir(components)
    	} else {
        	Ok(())
    	}
	}

	fn mkdir(&mut self, path: String) -> Result<()> {
		let mut components: Vec<&str> = path.split("/").collect();

		components.reverse();
		components.pop();

		self.traverse_mkdir(&mut components)
	}

	fn lsdir(&self, mut tabs: String) -> Result<()> {
		info!("{}{} ({:?})", tabs, self.get_name(), self.get_kind());

		tabs.push_str("  ");
		for (_, node) in self.children.iter() {
			if node.get_kind() == NodeKind::Directory {
				node.lsdir(tabs.clone())?;
			} else if node.get_kind() == NodeKind::File {
				info!("{}{} ({:?})", tabs, node.get_name(), node.get_kind());
			}
		}

		Ok(())
	}

	fn traverse_open(&mut self, components: &mut Vec<&str>, flags: OpenOptions) -> Result<Box<FileHandle>> {
		if let Some(component) = components.pop() {
				if components.is_empty() == true {
				// reach endpoint => reach file
				if flags.contains(OpenOptions::CREATE) {
					let file = &mut self.children.entry(String::from(component))
						.or_insert(Box::new(MemoryFsFile::new(String::from(component))));
						if file.get_kind() == NodeKind::File {
							file.get_handle(flags)
						} else {
							Err(Error::BadFsKind)
						}
				} else {
					if let Some(file) = self.children.get_mut(&String::from(component)) {
						file.get_handle(flags)
					} else {
						Err(Error::InvalidArgument)
					}
				}
			} else {
				if let Some(directory) = self.children.get_mut(&String::from(component)) {
					if directory.get_kind() != NodeKind::Directory {
						return Err(Error::BadFsKind);
					}

					directory.traverse_open(components, flags)
				} else {
					Err(Error::InvalidArgument)
				}
			}
		} else {
			Err(Error::InvalidArgument)
		}
	}

	fn open(&mut self, path: String, flags: OpenOptions) -> Result<Box<FileHandle>> {
		let mut components: Vec<&str> = path.split("/").collect();

		components.reverse();
		components.pop();

		self.traverse_open(&mut components, flags)
	}
}

#[derive(Debug, Clone)]
pub struct DataHandle(Arc<RwLock<Vec<u8>>>);

impl DataHandle {
    fn new() -> DataHandle {
        DataHandle(Arc::new(RwLock::new(Vec::new())))
    }
}

#[derive(Debug)]
pub struct MemoryFsFile {
	/// is the file write able?
	writeable: bool,
	/// Position within the file
	pos: usize,
	/// Director name
	name: String,
	/// File content
	data: DataHandle
}

impl MemoryFsFile {
	pub fn new(name: String) -> Self {
		MemoryFsFile {
			writeable: true,
			pos: 0,
			name: name,
			data: DataHandle::new()
		}
	}
}

impl VfsNode for MemoryFsFile {
	fn get_name(&self) -> String {
		self.name.clone()
	}

	fn get_kind(&self) -> NodeKind {
		NodeKind::File
	}

	fn get_handle(&self, opt: OpenOptions) -> Result<Box<FileHandle>> {
		Ok(Box::new(MemoryFsFile {
			writeable: opt.contains(OpenOptions::READWRITE),
			pos: self.pos,
			name: self.get_name(),
			data: self.data.clone()
		}))
	}
}

impl fmt::Write for MemoryFsFile {
	fn write_str(&mut self, s: &str) -> core::fmt::Result {
		if self.writeable == false {
			return Err(core::fmt::Error);
		}

		let mut guard = self.data.0.write();
		let ref mut vec: &mut Vec<u8> = guard.deref_mut();

		if self.pos + s.len() > vec.len() {
			vec.resize(self.pos + s.len(), 0);
		}

		vec[self.pos..self.pos + s.len()].clone_from_slice(s.as_bytes());
        self.pos += s.len();

		Ok(())
	}
}

impl FileHandle for MemoryFsFile {
	fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
		let guard = self.data.0.read();
		let ref vec: &Vec<u8> = guard.deref();

		if self.pos >= vec.len() {
			return Ok(0)
		}

		let len;
		if vec.len() - self.pos < buf.len() {
			len = vec.len() - self.pos
		} else {
			len = buf.len()
		}

		buf[0..len].clone_from_slice(&vec[self.pos..self.pos + len]);
		self.pos += len;

		Ok(len)
	}

	fn write(&mut self, buf: &[u8]) -> Result<usize> {
		if self.writeable == false {
			return Err(Error::BadFsPermission);
		}

		let mut guard = self.data.0.write();
		let ref mut vec: &mut Vec<u8> = guard.deref_mut();

		if self.pos + buf.len() > vec.len() {
			vec.resize(self.pos + buf.len(), 0);
		}

		vec[self.pos..self.pos + buf.len()].clone_from_slice(buf);
        self.pos += buf.len();

		Ok(buf.len())
	}
}

/// An in-memory file system
#[derive(Debug)]
pub struct MemoryFs {
    handle: SpinlockIrqSave<MemoryFsDirectory>,
}

impl MemoryFs {
    pub fn new() -> MemoryFs {
        MemoryFs {
			handle: SpinlockIrqSave::new(MemoryFsDirectory::new(String::from("/")))
		}
    }
}

impl Vfs for MemoryFs {
	fn mkdir(&mut self, path: String) -> Result<()> {
		self.handle.lock().mkdir(path)
	}

	fn lsdir(&self) -> Result<()> {
		self.handle.lock().lsdir(String::from(""))
	}

	fn open(&mut self, path: String, flags: OpenOptions) -> Result<Box<FileHandle>> {
		self.handle.lock().open(path, flags)
	}
}
