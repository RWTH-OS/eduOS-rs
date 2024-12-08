pub(crate) mod exit;
pub(crate) mod write;

use crate::syscall::exit::sys_exit;
use crate::syscall::write::sys_write;

/// number of the system call `exit`
pub const SYSNO_EXIT: usize = 0;

/// number of the system call `write`
pub const SYSNO_WRITE: usize = 1;

/// total number of system calls
pub const NO_SYSCALLS: usize = 2;

#[repr(align(64))]
#[repr(C)]
pub(crate) struct SyscallTable {
	handle: [*const usize; NO_SYSCALLS],
}

impl SyscallTable {
	pub const fn new() -> Self {
		SyscallTable {
			handle: [sys_exit as *const _, sys_write as *const _],
		}
	}
}

unsafe impl Send for SyscallTable {}
unsafe impl Sync for SyscallTable {}

impl Default for SyscallTable {
	fn default() -> Self {
		Self::new()
	}
}

pub(crate) static SYSHANDLER_TABLE: SyscallTable = SyscallTable::new();
