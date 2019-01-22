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

//! Implements a simple in-memory file system

use logging::*;
use errno::*;
use fs::{NodeKind, VfsNode, VfsNodeFile, VfsNodeDirectory, VfsNodeSymlink, Vfs,
		OpenOptions, FileHandle, SeekFrom, check_path};
use alloc::boxed::Box;
use alloc::vec::Vec;
use alloc::string::String;
use alloc::collections::BTreeMap;
use alloc::sync::Arc;
use core::ops::{Deref,DerefMut};
use core::any::Any;
use core::fmt;
use spin::RwLock;
use synch::spinlock::*;

#[derive(Debug)]
struct MemoryFsDirectory {
	/// in principle, a map with all entries of the current directory
	children: BTreeMap<String, Box<Any + core::marker::Send + core::marker::Sync>>
}

impl MemoryFsDirectory {
	pub fn new() -> Self {
		MemoryFsDirectory {
			children: BTreeMap::new()
		}
	}

	fn get_mut<T: VfsNode + Any>(&mut self, name: &String) -> Option<&mut T> {
		if let Some(b) = self.children.get_mut(name) {
			return b.downcast_mut::<T>();
		}
		None
	}

	fn get<T: VfsNode + Any>(&mut self, name: &String) -> Option<&T> {
		if let Some(b) = self.children.get_mut(name) {
			return b.downcast_ref::<T>();
		}
		None
	}
}

impl VfsNode for MemoryFsDirectory {
	/// Returns the node type
	fn get_kind(&self) -> NodeKind {
		NodeKind::Directory
	}
}

impl VfsNodeDirectory for MemoryFsDirectory {
	fn traverse_symlink(&mut self, components: &mut Vec<&str>, path: &String) -> Result<()> {
		if let Some(component) = components.pop() {
			let node_name = String::from(component);

			if components.is_empty() == true {
				// reach endpoint => reach link
				if self.children.get(&node_name).is_none() {
					// Create symlink
					let link = Box::new(MemoryFsSymlink::new(path));
					self.children.insert(node_name, link);

					Ok(())
				} else {
					Err(Error::InvalidArgument)
				}
			} else {
				// traverse to the directories to the endpoint
				if let Some(directory) = self.get_mut::<MemoryFsDirectory>(&node_name) {
					directory.traverse_symlink(components, path)
				} else {
					Err(Error::InvalidArgument)
				}
			}
		} else {
			Err(Error::InvalidArgument)
		}
	}

	fn traverse_mkdir(&mut self, components: &mut Vec<&str>) -> Result<()> {
		if let Some(component) = components.pop() {
			let node_name = String::from(component);

			{
				if let Some(directory) = self.get_mut::<MemoryFsDirectory>(&node_name) {
					return directory.traverse_mkdir(components);
				}
			}

			let mut directory = Box::new(MemoryFsDirectory::new());
			let result = directory.traverse_mkdir(components);
			self.children.insert(node_name, directory);

			result
		} else {
			Ok(())
		}
	}

	fn traverse_lsdir(&self, mut tabs: String) -> Result<()> {
		tabs.push_str("  ");
		for (name, node) in self.children.iter() {
			if let Some(directory) = node.downcast_ref::<MemoryFsDirectory>() {
				info!("{}{} ({:?})", tabs, name, self.get_kind());
				directory.traverse_lsdir(tabs.clone())?;
			} else if let Some(file) = node.downcast_ref::<MemoryFsFile>() {
				info!("{}{} ({:?})", tabs, name, file.get_kind());
			} else if let Some(link) = node.downcast_ref::<MemoryFsSymlink>() {
				info!("{}{} ({:?} -> {})", tabs, name, link.get_kind(), link.get_path());
			} else {
				info!("{}{} (Unknown))", tabs, name);
			}
		}

		Ok(())
	}

	fn traverse_open(&mut self, components: &mut Vec<&str>, flags: OpenOptions) -> Result<Box<FileHandle>> {
		if let Some(component) = components.pop() {
			let node_name = String::from(component);

			if components.is_empty() == true {
				// reach endpoint => reach file
				if let Some(file) = self.get_mut::<MemoryFsFile>(&node_name) {
					return file.get_handle(flags);
				}
			}

			if components.is_empty() == true {
				if flags.contains(OpenOptions::CREATE) {
					// Create file on demand
					let file = Box::new(MemoryFsFile::new());
					let result = file.get_handle(flags);
					self.children.insert(node_name, file);

					result
				} else {
					Err(Error::InvalidArgument)
				}
			} else {
				// traverse to the directories to the endpoint
				if let Some(directory) = self.get_mut::<MemoryFsDirectory>(&node_name) {
					directory.traverse_open(components, flags)
				} else {
					Err(Error::InvalidArgument)
				}
			}
		} else {
			Err(Error::InvalidArgument)
		}
	}
}

#[derive(Debug)]
struct MemoryFsSymlink {
	/// Path to the new location
	path: String
}

impl MemoryFsSymlink {
	pub fn new(path: &String) -> Self {
		MemoryFsSymlink {
			path: path.clone()
		}
	}
}

impl VfsNode for MemoryFsSymlink {
	fn get_kind(&self) -> NodeKind {
		NodeKind::Symlink
	}
}

impl VfsNodeSymlink for MemoryFsSymlink {
	fn get_path(&self) -> String {
		self.path.clone()
	}
}

#[derive(Debug, Clone)]
struct DataHandle(Arc<RwLock<Vec<u8>>>);

impl DataHandle {
	fn new() -> DataHandle {
		DataHandle(Arc::new(RwLock::new(Vec::new())))
	}
}

#[derive(Debug)]
struct MemoryFsFile {
	/// Is the file writeable?
	writeable: bool,
	/// Position within the file
	pos: Spinlock<usize>,
	/// File content
	data: DataHandle
}

impl MemoryFsFile {
	pub fn new() -> Self {
		MemoryFsFile {
			writeable: true,
			pos: Spinlock::new(0),
			data: DataHandle::new()
		}
	}
}

impl VfsNode for MemoryFsFile {
	fn get_kind(&self) -> NodeKind {
		NodeKind::File
	}
}

impl VfsNodeFile for MemoryFsFile {
	fn get_handle(&self, opt: OpenOptions) -> Result<Box<FileHandle>> {
		Ok(Box::new(MemoryFsFile {
			writeable: opt.contains(OpenOptions::READWRITE),
			pos: Spinlock::new(0),
			data: self.data.clone()
		}))
	}
}

impl Clone for MemoryFsFile {
	fn clone(&self) -> Self {
		MemoryFsFile {
			writeable: self.writeable,
			pos: Spinlock::new(*self.pos.lock()),
			data: self.data.clone()
		}
	}
}

impl fmt::Write for MemoryFsFile {
	fn write_str(&mut self, s: &str) -> core::fmt::Result {
		if self.writeable == false {
			return Err(core::fmt::Error);
		}

		let mut guard = self.data.0.write();
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
}

impl FileHandle for MemoryFsFile {
	fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
		let guard = self.data.0.read();
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

	fn write(&mut self, buf: &[u8]) -> Result<usize> {
		if self.writeable == false {
			return Err(Error::BadFsPermission);
		}

		let mut guard = self.data.0.write();
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

	fn seek(&mut self, style: SeekFrom) -> Result<u64> {
		let mut pos_guard = self.pos.lock();

		match style {
			SeekFrom::Start(n) => {
				*pos_guard = n as usize;
				Ok(n)
			}
			SeekFrom::End(n) => {
				let guard = self.data.0.read();
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
}

/// Entrypoint of the in-memory file system
#[derive(Debug)]
pub struct MemoryFs {
	handle: Spinlock<MemoryFsDirectory>,
}

impl MemoryFs {
	pub fn new() -> MemoryFs {
		MemoryFs {
			handle: Spinlock::new(MemoryFsDirectory::new())
		}
	}
}

impl Vfs for MemoryFs {
	fn mkdir(&mut self, path: &String) -> Result<()> {
		if check_path(path) {
			let mut components: Vec<&str> = path.split("/").collect();

			components.reverse();
			components.pop();

			self.handle.lock().traverse_mkdir(&mut components)
		} else {
			Err(Error::InvalidFsPath)
		}
	}

	fn lsdir(&self) -> Result<()> {
		info!("/");

		self.handle.lock().traverse_lsdir(String::from(""))
	}

	fn open(&mut self, path: &String, flags: OpenOptions) -> Result<Box<FileHandle>> {
		if check_path(path) {
			let mut components: Vec<&str> = path.split("/").collect();

			components.reverse();
			components.pop();

			self.handle.lock().traverse_open(&mut components, flags)
		} else {
			Err(Error::InvalidFsPath)
		}
	}

	fn symlink(&mut self, path1: &String, path2: &String) -> Result<()> {
		if check_path(path2) {
			let mut components: Vec<&str> = path2.split("/").collect();

			components.reverse();
			components.pop();

			self.handle.lock().traverse_symlink(&mut components, path1)
		} else {
			Err(Error::InvalidFsPath)
		}
	}
}
