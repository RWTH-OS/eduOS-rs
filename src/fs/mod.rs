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

//! Definition a simple virtual file system

#![allow(dead_code)]

mod initrd;

use logging::*;
use errno::*;
use fs::initrd::MemoryFs;
use alloc::vec::Vec;
use alloc::boxed::Box;
use alloc::string::String;

/// Type of the VfsNode
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum NodeKind {
	/// Node represent a file
	File,
	/// Node represent a directory
	Directory
}

bitflags! {
	/// Options for opening files
    pub struct OpenOptions: u32 {
		/// Open file for reading.
        const READONLY  = 0b00000001;
		/// Open file for reading and writing.
        const READWRITE = 0b00000010;
		/// File is created if it does not exist
		const CREATE    = 0b00000100;
    }
}

/// VfsNode represents an internal node of the virtual file system.
trait VfsNode: core::fmt::Debug + core::marker::Send + core::marker::Sync {
	/// Name of the current node
	fn get_name(&self) -> String;

	/// Determines the current node type
	fn get_kind(&self) -> NodeKind;
}

/// VfsNodeFile represents a file node of the virtual file system.
trait VfsNodeFile: VfsNode + core::fmt::Debug + core::marker::Send + core::marker::Sync {
	/// Create a file handle to the current file
	fn get_handle(&self, _opt: OpenOptions) -> Result<Box<FileHandle>>;
}

/// VfsNodeDirectory represents a directory node of the virtual file system.
trait VfsNodeDirectory: VfsNode + core::fmt::Debug + core::marker::Send + core::marker::Sync {
	/// Create a directory node at the location `path`.
	fn mkdir(&mut self, _path: &String) -> Result<()>;

	fn traverse_mkdir(&mut self, _components: &mut Vec<&str>) -> Result<()>;

	/// Helper function to print the current state of the file system
	fn lsdir(&self, _tabs: String) -> Result<()>;

	fn traverse_open(&mut self, _components: &mut Vec<&str>, _flags: OpenOptions) -> Result<Box<FileHandle>>;

	/// Open a file node with the path `path`.
	/// `path` must be an absolute path to the file, while `flags` defined
	/// if the file is writeable or created on demand.
	fn open(&mut self, _path: &String, _flags: OpenOptions) -> Result<Box<FileHandle>>;
}

/// The trait `Vfs` specifies all operation on the virtual file systems.
trait Vfs: core::fmt::Debug + core::marker::Send + core::marker::Sync {
	/// Create a directory node at the location `path`.
	fn mkdir(&mut self, path: &String) -> Result<()>;

	/// Print the current state of the file system
	fn lsdir(&self) -> Result<()>;

	/// Open a file with the path `path`.
	/// `path` must be an absolute path to the file, while `flags` defined
	/// if the file is writeable or created on demand.
	fn open(&mut self, path: &String, flags: OpenOptions) -> Result<Box<FileHandle>>;
}

/// Enumeration of possible methods to seek within an I/O object.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum SeekFrom {
	/// Set the offset to the provided number of bytes.
	Start(u64),
	/// Set the offset to the size of this object plus the specified number of bytes.
	///
    /// It is possible to seek beyond the end of an object, but it's an error to
    /// seek before byte 0.
	End(i64),
	/// Set the offset to the current position plus the specified number of bytes.
	///
    /// It is possible to seek beyond the end of an object, but it's an error to
    /// seek before byte 0.
	Current(i64)
}

/// The trait `FileHandle` defines all functions hat can be applied to the file.
pub trait FileHandle: core::fmt::Debug + core::fmt::Write {
	fn read(&mut self, buf: &mut [u8]) -> Result<usize>;
	fn write(&mut self, buf: &[u8]) -> Result<usize>;
	fn seek(&mut self, style: SeekFrom) -> Result<u64>;
}

/// Entrypoint of the file system
static mut VFS_ROOT: Option<MemoryFs> = None;

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
pub fn open(path: &String, flags: OpenOptions) -> Result<Box<FileHandle>> {
	unsafe { VFS_ROOT.as_mut().unwrap().open(path, flags) }
}

/// Help function to check if the argument is an abolute path
fn check_path(path: &String) -> bool{
	if let Some(pos) = path.find('/') {
		if pos == 0 {
			return true;
		}
	}

	false
}

pub fn init() {
	let mut root = MemoryFs::new();

	root.mkdir(&String::from("/bin")).unwrap();
	root.mkdir(&String::from("/dev")).unwrap();

	//root.lsdir().unwrap();
	//info!("root {:?}", root);
	unsafe {
		VFS_ROOT = Some(root);
	}
}
