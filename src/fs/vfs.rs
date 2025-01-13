//! Implements a simple virtual file system

use crate::errno::*;
use crate::fd::OpenOption;
use crate::fd::{FileStatus, IoInterface};
use crate::fs::initrd::{RamHandle, RomHandle};
use crate::fs::{check_path, NodeKind, SeekFrom, Vfs, VfsNode, VfsNodeDirectory, VfsNodeFile};
use crate::io;
use crate::logging::*;
use crate::synch::spinlock::*;
use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec::Vec;
use core::any::Any;
use core::fmt;

#[derive(Debug)]
struct VfsDirectory {
	/// in principle, a map with all entries of the current directory
	children: BTreeMap<String, Box<dyn Any + core::marker::Send + core::marker::Sync>>,
}

impl VfsDirectory {
	pub fn new() -> Self {
		VfsDirectory {
			children: BTreeMap::new(),
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

impl VfsNode for VfsDirectory {
	/// Returns the node type
	fn get_kind(&self) -> NodeKind {
		NodeKind::Directory
	}
}

impl VfsNodeDirectory for VfsDirectory {
	fn traverse_mkdir(&mut self, components: &mut Vec<&str>) -> Result<()> {
		if let Some(component) = components.pop() {
			let node_name = String::from(component);

			{
				if let Some(directory) = self.get_mut::<VfsDirectory>(&node_name) {
					return directory.traverse_mkdir(components);
				}
			}

			let mut directory = Box::new(VfsDirectory::new());
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
			if let Some(directory) = node.downcast_ref::<VfsDirectory>() {
				info!("{}{} ({:?})", tabs, name, self.get_kind());
				directory.traverse_lsdir(tabs.clone())?;
			} else if let Some(file) = node.downcast_ref::<VfsFile>() {
				info!("{}{} ({:?})", tabs, name, file.get_kind());
			} else {
				info!("{}{} (Unknown))", tabs, name);
			}
		}

		Ok(())
	}

	fn traverse_open(
		&mut self,
		components: &mut Vec<&str>,
		flags: OpenOption,
	) -> Result<Arc<dyn IoInterface>> {
		if let Some(component) = components.pop() {
			let node_name = String::from(component);

			if components.is_empty() == true {
				// reach endpoint => reach file
				if let Some(file) = self.get_mut::<VfsFile>(&node_name) {
					return file.get_handle(flags);
				}
			}

			if components.is_empty() == true {
				if flags.contains(OpenOption::O_CREAT) {
					// Create file on demand
					let file = Box::new(VfsFile::new());
					let result = file.get_handle(flags);
					self.children.insert(node_name, file);

					result
				} else {
					Err(Error::InvalidArgument)
				}
			} else {
				// traverse to the directories to the endpoint
				if let Some(directory) = self.get_mut::<VfsDirectory>(&node_name) {
					directory.traverse_open(components, flags)
				} else {
					Err(Error::InvalidArgument)
				}
			}
		} else {
			Err(Error::InvalidArgument)
		}
	}

	fn traverse_mount(&mut self, components: &mut Vec<&str>, slice: &'static [u8]) -> Result<()> {
		if let Some(component) = components.pop() {
			let node_name = String::from(component);

			if components.is_empty() == true {
				// Create file on demand
				let file = Box::new(VfsFile::new_from_rom(slice));
				self.children.insert(node_name, file);

				Ok(())
			} else {
				// traverse to the directories to the endpoint
				if let Some(directory) = self.get_mut::<VfsDirectory>(&node_name) {
					directory.traverse_mount(components, slice)
				} else {
					Err(Error::InvalidArgument)
				}
			}
		} else {
			Err(Error::InvalidArgument)
		}
	}
}

/// Enumeration of possible methods to seek within an I/O object.
#[derive(Debug, Clone)]
enum DataHandle {
	RAM(RamHandle),
	ROM(RomHandle),
}

#[derive(Debug, Clone)]
struct VfsFile {
	/// File content
	data: DataHandle,
}

impl VfsFile {
	pub fn new() -> Self {
		VfsFile {
			data: DataHandle::RAM(RamHandle::new(true)),
		}
	}

	pub fn new_from_rom(slice: &'static [u8]) -> Self {
		VfsFile {
			data: DataHandle::ROM(RomHandle::new(slice)),
		}
	}
}

impl VfsNode for VfsFile {
	fn get_kind(&self) -> NodeKind {
		NodeKind::File
	}
}

impl VfsNodeFile for VfsFile {
	fn get_handle(&self, opt: OpenOption) -> Result<Arc<dyn IoInterface>> {
		match self.data {
			DataHandle::RAM(ref data) => Ok(Arc::new(VfsFile {
				data: DataHandle::RAM(data.get_handle(opt)),
			})),
			DataHandle::ROM(ref data) => Ok(Arc::new(VfsFile {
				data: DataHandle::ROM(data.get_handle(opt)),
			})),
		}
	}
}

impl fmt::Write for VfsFile {
	fn write_str(&mut self, s: &str) -> core::fmt::Result {
		match self.data {
			DataHandle::RAM(ref mut data) => data.write_str(s),
			_ => Err(core::fmt::Error),
		}
	}
}

impl IoInterface for VfsFile {
	fn read(&self, buf: &mut [u8]) -> io::Result<usize> {
		match self.data {
			DataHandle::RAM(ref data) => data.read(buf),
			DataHandle::ROM(ref data) => data.read(buf),
		}
	}

	fn write(&self, buf: &[u8]) -> io::Result<usize> {
		match self.data {
			DataHandle::RAM(ref data) => data.write(buf),
			_ => Err(io::Error::EBADF),
		}
	}

	fn seek(&self, style: SeekFrom) -> io::Result<usize> {
		match self.data {
			DataHandle::RAM(ref data) => data.seek(style),
			DataHandle::ROM(ref data) => data.seek(style),
		}
	}

	fn fstat(&self) -> io::Result<FileStatus> {
		let file_size = match self.data {
			DataHandle::RAM(ref data) => data.len(),
			DataHandle::ROM(ref data) => data.len(),
		};

		Ok(FileStatus { file_size })
	}
}

/// Entrypoint of the in-memory file system
#[derive(Debug)]
pub(crate) struct Fs {
	handle: Spinlock<VfsDirectory>,
}

impl Fs {
	pub fn new() -> Fs {
		Fs {
			handle: Spinlock::new(VfsDirectory::new()),
		}
	}
}

impl Vfs for Fs {
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

	fn open(&mut self, path: &str, flags: OpenOption) -> Result<Arc<dyn IoInterface>> {
		if check_path(path) {
			let mut components: Vec<&str> = path.split("/").collect();

			components.reverse();
			components.pop();

			self.handle.lock().traverse_open(&mut components, flags)
		} else {
			Err(Error::InvalidFsPath)
		}
	}

	/// Mound memory region as file
	fn mount(&mut self, path: &String, slice: &'static [u8]) -> Result<()> {
		if check_path(path) {
			let mut components: Vec<&str> = path.split("/").collect();

			components.reverse();
			components.pop();

			self.handle.lock().traverse_mount(&mut components, slice)
		} else {
			Err(Error::InvalidFsPath)
		}
	}
}
