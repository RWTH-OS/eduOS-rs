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

//! Architecture dependent interface to initialize a task

use core::mem::size_of;
use scheduler::task::*;
use scheduler::{do_exit,get_current_taskid};
use consts::*;
use rlibc::*;
use logging::*;

#[derive(Debug)]
#[repr(C, packed)]
struct State {
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
	rip: u64
}

extern "C" fn leave_task() {
	debug!("finish task {}", get_current_taskid());

	do_exit();

	loop {}
}

impl TaskFrame for Task {
    fn create_stack_frame(&mut self, func: extern fn())
	{
		unsafe {
			let mut stack: *mut u64 = (self.stack.top() - 16) as *mut u64;

			memset(self.stack.bottom() as *mut u8, 0xCD, KERNEL_STACK_SIZE);

			/* Only marker for debugging purposes, ... */
			*stack = 0xDEADBEEFu64;
			stack = (stack as usize - size_of::<u64>()) as *mut u64;

			/* the first-function-to-be-called's arguments, ... */
			//TODO: add arguments

			/* and the "caller" we shall return to.
	 		 * This procedure cleans the task after exit. */
			*stack = (leave_task as *const()) as u64;
			stack = (stack as usize - size_of::<State>()) as *mut u64;

			let state: *mut State = stack as *mut State;
			memset(state as *mut u8, 0x00, size_of::<State>());

			(*state).rsp = (stack as usize + size_of::<State>()) as u64;
			(*state).rbp = (*state).rsp + size_of::<u64>() as u64;

			(*state).rip = (func as *const()) as u64;;
			(*state).rflags = 0x1002u64;

			/* Set the task's stack pointer entry to the stack we have crafted right now. */
			self.last_stack_pointer =  stack as u64;
		}
	}
}
