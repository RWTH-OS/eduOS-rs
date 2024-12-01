//! Basic error handling

use core::{fmt, result};

pub type Result<T> = result::Result<T, Error>;

/// Possible errors of eduOS-rs
#[derive(Debug, Clone)]
pub enum Error {
	/// Usage of a invalid priority
	BadPriority,
}

impl fmt::Display for Error {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match *self {
			Error::BadPriority => write!(f, "Invalid priority number"),
		}
	}
}
