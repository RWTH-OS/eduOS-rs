//! Definition a simple virtual file system

#![allow(dead_code)]

mod initrd;
mod vfs;

use crate::errno::*;
use crate::fd::{self, FileDescriptor, OpenOption};
use crate::fd::{IoInterface, SeekFrom};
use crate::fs::vfs::Fs;
use crate::io;
use crate::logging::*;
use crate::scheduler::{insert_io_interface, remove_io_interface};
use alloc::string::{String, ToString};
use alloc::sync::Arc;
use alloc::vec::Vec;
use core::include_bytes;

static DEMO: &[u8] = include_bytes!("../../demo/hello");

/// Type of the VfsNode
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) enum NodeKind {
	/// Node represent a file
	File,
	/// Node represent a directory
	Directory,
}

/// VfsNode represents an internal node of the virtual file system.
trait VfsNode: core::fmt::Debug + core::marker::Send + core::marker::Sync {
	/// Determines the current node type
	fn get_kind(&self) -> NodeKind;
}

/// VfsNodeFile represents a file node of the virtual file system.
trait VfsNodeFile: VfsNode + core::fmt::Debug + core::marker::Send + core::marker::Sync {
	/// Create an IO interface to the current file
	fn get_handle(&self, _opt: OpenOption) -> Result<Arc<dyn IoInterface>>;
}

/// VfsNodeDirectory represents a directory node of the virtual file system.
trait VfsNodeDirectory: VfsNode + core::fmt::Debug + core::marker::Send + core::marker::Sync {
	/// Helper function to create a new dirctory node
	fn traverse_mkdir(&mut self, _components: &mut Vec<&str>) -> Result<()>;

	/// Helper function to print the current state of the file system
	fn traverse_lsdir(&self, _tabs: String) -> Result<()>;

	/// Helper function to open a file
	fn traverse_open(
		&mut self,
		_components: &mut Vec<&str>,
		_flags: OpenOption,
	) -> Result<Arc<dyn IoInterface>>;

	/// Mound memory region as file
	fn traverse_mount(&mut self, _components: &mut Vec<&str>, slice: &'static [u8]) -> Result<()>;
}

/// The trait `Vfs` specifies all operation on the virtual file systems.
trait Vfs: core::fmt::Debug + core::marker::Send + core::marker::Sync {
	/// Create a directory node at the location `path`.
	fn mkdir(&mut self, path: &String) -> Result<()>;

	/// Print the current state of the file system
	fn lsdir(&self) -> Result<()>;

	/// Open a file with the path `path`.
	/// `path` must be an absolute path to the file, while `flags` defined
	fn open(&mut self, path: &str, flags: OpenOption) -> Result<Arc<dyn IoInterface>>;

	/// Mound memory region as file
	fn mount(&mut self, path: &String, slice: &'static [u8]) -> Result<()>;
}

/// Entrypoint of the file system
static mut VFS_ROOT: Option<Fs> = None;

/// List the current state of file system
pub fn lsdir() -> Result<()> {
	unsafe { VFS_ROOT.as_mut().unwrap().lsdir() }
}

/// Create a directory with the path `path`.
/// `path` must be a absolete path to the direcory.
pub fn mkdir(path: &String) -> Result<()> {
	unsafe { VFS_ROOT.as_mut().unwrap().mkdir(path) }
}

/// Open a file with the path `path`.
/// `path` must be an absolute path to the file, while `flags` defined
/// if the file is writeable or created on demand.
pub fn open(name: &str, flags: OpenOption) -> io::Result<FileDescriptor> {
	debug!("Open {}, {:?}", name, flags);

	let fs = unsafe { VFS_ROOT.as_mut().unwrap() };
	if let Ok(file) = fs.open(name, flags) {
		let fd = insert_io_interface(file)?;
		Ok(fd)
	} else {
		Err(io::Error::EINVAL)
	}
}

/// Mount slice to to `path`
pub fn mount(path: &String, slice: &'static [u8]) -> Result<()> {
	unsafe { VFS_ROOT.as_mut().unwrap().mount(path, slice) }
}

/// Help function to check if the argument is an abolute path
fn check_path(path: &str) -> bool {
	if let Some(pos) = path.find('/') {
		if pos == 0 {
			return true;
		}
	}

	false
}

#[derive(Debug)]
pub struct File {
	fd: FileDescriptor,
	path: String,
}

impl File {
	/// Attempts to create a file in read-write mode.
	pub fn create(path: &str) -> io::Result<Self> {
		let fd = open(path, OpenOption::O_RDWR | OpenOption::O_CREAT)?;

		Ok(File {
			fd,
			path: path.to_string(),
		})
	}

	/// Attempts to open a file in read-write mode.
	pub fn open(path: &str) -> io::Result<Self> {
		let fd = open(path, OpenOption::O_RDWR)?;

		Ok(File {
			fd,
			path: path.to_string(),
		})
	}

	pub fn len(&self) -> io::Result<usize> {
		let fstat = fd::fstat(self.fd)?;
		Ok(fstat.file_size)
	}
}

impl crate::io::Read for File {
	fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
		fd::read(self.fd, buf)
	}
}

impl crate::io::Write for File {
	fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
		fd::write(self.fd, buf)
	}
}

impl Drop for File {
	fn drop(&mut self) {
		let _ = remove_io_interface(self.fd);
	}
}

pub(crate) fn init() {
	let mut root = Fs::new();

	root.mkdir(&String::from("/bin")).unwrap();
	root.mkdir(&String::from("/dev")).unwrap();

	if DEMO.len() > 0 {
		info!(
			"Found mountable file at 0x{:x} (len 0x{:x})",
			DEMO.as_ptr() as u64,
			DEMO.len()
		);
		root.mount(&String::from("/bin/demo"), &DEMO)
			.expect("Unable to mount file");
	}

	root.lsdir().unwrap();
	//info!("root {:?}", root);
	unsafe {
		VFS_ROOT = Some(root);
	}
}
