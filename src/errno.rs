// Copyright (c) 2017-2018 Stefan Lankes, RWTH Aachen University
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

//! Basic error handling

use core::{fmt, result};

pub type Result<T> = result::Result<T, Error>;

/// Possible errors of eduOS-rs
#[derive(Debug, Clone)]
pub enum Error {
	/// Usage of a invalid priority
	BadPriority,
	BadFsKind,
	BadFsOperation,
	BadFsPermission,
	InvalidFsPath,
	InvalidArgument,
}

impl fmt::Display for Error {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match *self {
			Error::BadPriority => write!(f, "Invalid priority number"),
			Error::BadFsKind => write!(f, "Bad file system kind"),
			Error::BadFsOperation => write!(f, "Bad file system operation"),
			Error::BadFsPermission => write!(f, "Bad file permission"),
			Error::InvalidFsPath => write!(f, "Invalid file system path"),
			Error::InvalidArgument => write!(f, "Inavlid argument"),
		}
	}
}
