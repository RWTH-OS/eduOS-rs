// Copyright (c) 2017 Stefan Lankes, RWTH Aachen University
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

pub mod write;
pub mod exit;

use syscall::exit::sys_exit;
use syscall::write::sys_write;

/// number of the system call `exit`
pub const SYSNO_EXIT: usize = 0;

/// number of the system call `write`
pub const SYSNO_WRITE: usize = 1;

/// total number of system calls
pub const NO_SYSCALLS: usize = 2;

#[repr(align(64))]
#[repr(C)]
pub struct SyscallTable{
	 handle: [*const usize; NO_SYSCALLS]
}

impl SyscallTable {
	pub const fn new() -> Self {
		SyscallTable {
			handle:	[sys_exit as *const _,
					 sys_write as *const _]
		}
	}
}

unsafe impl Send for SyscallTable {}
unsafe impl Sync for SyscallTable {}

#[no_mangle]
pub static SYSHANDLER_TABLE: SyscallTable = SyscallTable::new();
