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

#[cfg(target_arch="x86_64")]
#[derive(Debug)]
#[repr(C, packed)]
pub struct State {
	/// R15 register
	pub r15: u64,
	/// R14 register
	pub r14: u64,
	/// R13 register
	pub r13: u64,
	/// R12 register
	pub r12: u64,
	/// R11 register
	pub r11: u64,
	/// R10 register
	pub r10: u64,
	/// R9 register
	pub r9: u64,
	/// R8 register
	pub r8: u64,
	/// RDI register
	pub rdi: u64,
	/// RSI register
	pub rsi: u64,
	/// RBP register
	pub rbp: u64,
	/// (pseudo) RSP register
	pub rsp: u64,
	/// RBX register
	pub rbx: u64,
	/// RDX register
	pub rdx: u64,
	/// RCX register
	pub rcx: u64,
	/// RAX register
	pub rax: u64,
	/// interrupt number
	pub int_no: u64,

	// pushed by the processor automatically

	/// error code of the exception
	pub error: u64,
	/// instruction pointer
	pub ip: u64,
	/// code selector
	pub cs: u64,
	/// status flags
	pub rflags: u64,
	/// user-space stack pointer
	pub userrsp: u64,
	/// stack selector
	pub ss: u64
}

#[cfg(target_arch="x86")]
#[derive(Debug)]
#[repr(C, packed)]
pub struct State {
	/// ds register
	pub ds: u32,
	/// es register
	pub es: u32,
	/// EDI register
	pub edi: u32,
	/// ESI register
	pub esi: u32,
	/// EBP register
	pub ebp: u32,
	/// ESP register
	pub esp: u32,
	/// EBX register
	pub ebx: u32,
	/// EDX register
	pub edx: u32,
	/// ECX register
	pub ecx: u32,
	/// EAX register
	pub eax: u32,		/* pushed by 'pusha' */

	/// Interrupt number
	pub int_no: u32,

	// pushed by the processor automatically
	pub error: u32,
	pub ip: u32,
	pub cs: u32,
	pub eflags: u32,
	pub useresp: u32,
	pub ss: u32,
}

extern "C" fn leave_task() {
	debug!("finish task {}", get_current_taskid());

	do_exit();

	loop {}
}

impl TaskFrame for Task {
	#[cfg(target_arch="x86_64")]
    fn create_stack_frame(&mut self, func: extern fn())
	{
		unsafe {
			let mut stack: *mut u64 = ((*self.stack).top() - 16) as *mut u64;

			memset((*self.stack).bottom() as *mut u8, 0xCD, KERNEL_STACK_SIZE);

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
			// we elimante frame pointers => no setting rbp required
			//(*state).rbp = (*state).rsp + size_of::<u64>() as u64;

			(*state).int_no = 0xB16B00B5u64;
			(*state).error =  0xC03DB4B3u64;

			(*state).ip = (func as *const()) as u64;;
			(*state).cs = 0x08;
			(*state).ss = 0x10;
			(*state).rflags = 0x1202u64;
			(*state).userrsp = (*state).rsp;

			/* Set the task's stack pointer entry to the stack we have crafted right now. */
			self.last_stack_pointer = stack as usize;
		}
	}

	#[cfg(target_arch="x86")]
    fn create_stack_frame(&mut self, func: extern fn())
	{
		unsafe {
			let mut stack: *mut u32 = ((*self.stack).top() - 16) as *mut u32;

			memset((*self.stack).bottom() as *mut u8, 0xCD, KERNEL_STACK_SIZE);

			/* Only marker for debugging purposes, ... */
			*stack = 0xDEADBEEFu32;
			stack = (stack as usize - size_of::<u32>()) as *mut u32;

			/* the first-function-to-be-called's arguments, ... */
			//TODO: add arguments

			/* and the "caller" we shall return to.
	 		 * This procedure cleans the task after exit. */
			*stack = (leave_task as *const()) as u32;
			let state_size = size_of::<State>() - 2*size_of::<u32>();
			stack = (stack as usize - state_size) as *mut u32;

			let state: *mut State = stack as *mut State;
			memset(state as *mut u8, 0x00, state_size);

			(*state).esp = (stack as usize + state_size) as u32;
			// we elimante frame pointers => no setting ebp required
			//(*state).ebp = (*state).esp + size_of::<u32>() as u32;

			(*state).int_no = 0xB16B00B5u32;
			(*state).error =  0xC03DB4B3u32;

			(*state).ip = (func as *const()) as u32;
			(*state).cs = 0x08;
			(*state).ds = 0x10;
			(*state).es = 0x10;
			(*state).eflags = 0x1202u32;

			/* Set the task's stack pointer entry to the stack we have crafted right now. */
			self.last_stack_pointer = stack as usize;
		}
	}
}
