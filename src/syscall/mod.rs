// Copyright (c) 2017 Stefan Lankes, RWTH Aachen University
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

mod exit;
mod invalid;
mod nothing;
mod write;

use crate::syscall::exit::sys_exit;
use crate::syscall::invalid::sys_invalid;
use crate::syscall::nothing::sys_nothing;
use crate::syscall::write::{sys_write, sys_writev};

/// number of the system call `write`
pub const SYSNO_WRITE: usize = 1;

/// number of the system call `close`
pub const SYSNO_CLOSE: usize = 3;

pub const SYSNO_IOCTL: usize = 16;

pub const SYSNO_WRITEV: usize = 20;

/// number of the system call `exit`
pub const SYSNO_EXIT: usize = 60;

pub const SYSNO_ARCH_PRCTL: usize = 158;

/// set pointer to thread ID
pub const SYSNO_SET_TID_ADDRESS: usize = 218;

/// exit all threads in a process
pub const SYSNO_EXIT_GROUP: usize = 231;

/// total number of system calls
pub const NO_SYSCALLS: usize = 400;

#[repr(align(64))]
#[repr(C)]
pub struct SyscallTable {
	handle: [*const usize; NO_SYSCALLS],
}

impl SyscallTable {
	pub const fn new() -> Self {
		let mut table = SyscallTable {
			handle: [sys_invalid as *const _; NO_SYSCALLS],
		};

		table.handle[SYSNO_WRITE] = sys_write as *const _;
		table.handle[SYSNO_CLOSE] = sys_nothing as *const _;
		table.handle[SYSNO_IOCTL] = sys_nothing as *const _;
		table.handle[SYSNO_WRITEV] = sys_writev as *const _;
		table.handle[SYSNO_EXIT] = sys_exit as *const _;
		table.handle[SYSNO_ARCH_PRCTL] = sys_nothing as *const _;
		table.handle[SYSNO_SET_TID_ADDRESS] = sys_nothing as *const _;
		table.handle[SYSNO_EXIT_GROUP] = sys_exit as *const _;

		table
	}
}

unsafe impl Send for SyscallTable {}
unsafe impl Sync for SyscallTable {}

#[no_mangle]
pub static SYSHANDLER_TABLE: SyscallTable = SyscallTable::new();
