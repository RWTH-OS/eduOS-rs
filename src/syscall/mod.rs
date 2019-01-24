// Copyright (c) 2017 Stefan Lankes, RWTH Aachen University
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

mod write;
mod exit;
mod invalid;
mod nothing;

use syscall::exit::sys_exit;
use syscall::write::{sys_write,sys_writev};
use syscall::invalid::sys_invalid;
use syscall::nothing::sys_nothing;

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
pub struct SyscallTable{
	 handle: [*const usize; NO_SYSCALLS]
}

impl SyscallTable {
	pub const fn new() -> Self {
		let mut table = SyscallTable {
			handle:	[sys_invalid as *const _; NO_SYSCALLS]
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
