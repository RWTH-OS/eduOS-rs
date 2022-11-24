// Copyright (c) 2019 Stefan Lankes, RWTH Aachen University
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

//! Definition a simple virtual file system

#![allow(dead_code)]

mod initrd;
mod vfs;

use crate::errno::*;
use crate::fs::vfs::Fs;
use crate::logging::*;
use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;
use core::convert::TryInto;
use core::include_bytes;

static DEMO: &[u8] = include_bytes!("../../demo/hello");

/// Type of the VfsNode
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum NodeKind {
	/// Node represent a file
	File,
	/// Node represent a directory
	Directory,
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
	/// Determines the current node type
	fn get_kind(&self) -> NodeKind;
}

/// VfsNodeFile represents a file node of the virtual file system.
trait VfsNodeFile: VfsNode + core::fmt::Debug + core::marker::Send + core::marker::Sync {
	/// Create a file handle to the current file
	fn get_handle(&self, _opt: OpenOptions) -> Result<Box<dyn FileHandle>>;
}

/// VfsNodeFile represents a file node of the virtual file system.
trait VfsNodeSymlink: VfsNode + core::fmt::Debug + core::marker::Send + core::marker::Sync {
	/// Retuns the path to the new location
	fn get_path(&self) -> String;
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
		_flags: OpenOptions,
	) -> Result<Box<dyn FileHandle>>;

	/// Mound memory region as file
	fn traverse_mount(&mut self, _components: &mut Vec<&str>, addr: u64, len: u64) -> Result<()>;
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
	fn open(&mut self, path: &String, flags: OpenOptions) -> Result<Box<dyn FileHandle>>;

	/// Mound memory region as file
	fn mount(&mut self, path: &String, addr: u64, len: u64) -> Result<()>;
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
	Current(i64),
}

/// The trait `FileHandle` defines all functions hat can be applied to the file.
pub trait FileHandle: core::fmt::Debug + core::fmt::Write {
	fn read(&mut self, buf: &mut [u8]) -> Result<usize>;
	fn write(&mut self, buf: &[u8]) -> Result<usize>;
	fn seek(&mut self, style: SeekFrom) -> Result<u64>;
	fn len(&self) -> usize;
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
pub fn open(path: &String, flags: OpenOptions) -> Result<Box<dyn FileHandle>> {
	unsafe { VFS_ROOT.as_mut().unwrap().open(path, flags) }
}

/// A symbolic link `path2` is created to `path1`
pub fn mount(path: &String, addr: u64, len: u64) -> Result<()> {
	unsafe { VFS_ROOT.as_mut().unwrap().mount(path, addr, len) }
}

/// Help function to check if the argument is an abolute path
fn check_path(path: &String) -> bool {
	if let Some(pos) = path.find('/') {
		if pos == 0 {
			return true;
		}
	}

	false
}

pub fn init() {
	let mut root = Fs::new();

	root.mkdir(&String::from("/bin")).unwrap();
	root.mkdir(&String::from("/dev")).unwrap();

	if DEMO.len() > 0 {
		info!(
			"Found mountable file at 0x{:x} (len 0x{:x})",
			&DEMO as *const _ as u64,
			DEMO.len()
		);
		root.mount(
			&String::from("/bin/demo"),
			DEMO.as_ptr() as u64,
			DEMO.len().try_into().unwrap(),
		)
		.expect("Unable to mount file");
	}

	//root.lsdir().unwrap();
	//info!("root {:?}", root);
	unsafe {
		VFS_ROOT = Some(root);
	}
}
