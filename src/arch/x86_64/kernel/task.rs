// Copyright (c) 2017-2018 Stefan Lankes, RWTH Aachen University
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

//! Architecture dependent interface to initialize a task

use crate::consts::*;
use crate::logging::*;
use crate::scheduler::task::*;
use crate::scheduler::{do_exit, get_current_taskid};
use core::mem::size_of;
use core::ptr::write_bytes;

#[repr(C, packed)]
struct State {
	/// GS register
	gs: u64,
	/// FS register
	fs: u64,
	/// R15 register
	r15: u64,
	/// R14 register
	r14: u64,
	/// R13 register
	r13: u64,
	/// R12 register
	r12: u64,
	/// R11 register
	r11: u64,
	/// R10 register
	r10: u64,
	/// R9 register
	r9: u64,
	/// R8 register
	r8: u64,
	/// RDI register
	rdi: u64,
	/// RSI register
	rsi: u64,
	/// RBP register
	rbp: u64,
	/// (pseudo) RSP register
	rsp: u64,
	/// RBX register
	rbx: u64,
	/// RDX register
	rdx: u64,
	/// RCX register
	rcx: u64,
	/// RAX register
	rax: u64,
	/// status flags
	rflags: u64,
	/// instruction pointer
	rip: u64,
}

extern "C" fn leave_task() -> ! {
	debug!("finish task {}", get_current_taskid());

	do_exit();
}

impl TaskFrame for Task {
	fn create_stack_frame(&mut self, func: extern "C" fn()) {
		unsafe {
			let mut stack: *mut u64 = ((*self.stack).top()) as *mut u64;

			write_bytes((*self.stack).bottom() as *mut u8, 0xCD, STACK_SIZE);

			/* Only marker for debugging purposes, ... */
			*stack = 0xDEADBEEFu64;
			stack = (stack as usize - size_of::<u64>()) as *mut u64;

			/* the first-function-to-be-called's arguments, ... */
			//TODO: add arguments

			/* and the "caller" we shall return to.
			 * This procedure cleans the task after exit. */
			*stack = (leave_task as *const ()) as u64;
			stack = (stack as usize - size_of::<State>()) as *mut u64;

			let state: *mut State = stack as *mut State;
			write_bytes(state, 0x00, 1);

			(*state).rsp = (stack as usize + size_of::<State>()) as u64;
			(*state).rbp = (*state).rsp + size_of::<u64>() as u64;
			(*state).gs = ((*self.stack).top()) as u64;

			(*state).rip = (func as *const ()) as u64;
			(*state).rflags = 0x1202u64;

			/* Set the task's stack pointer entry to the stack we have crafted right now. */
			self.last_stack_pointer = stack as usize;
		}
	}
}
