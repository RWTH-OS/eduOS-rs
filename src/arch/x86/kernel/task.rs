//! Architecture dependent interface to initialize a task

use crate::arch::mm::VirtAddr;
use crate::consts::*;
use crate::logging::*;
use crate::scheduler::task::*;
use crate::scheduler::{do_exit, get_current_taskid};
use core::mem::size_of;
use core::ptr::write_bytes;

#[cfg(target_arch = "x86_64")]
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

#[cfg(target_arch = "x86")]
#[repr(C, packed)]
struct State {
	/// EDI register
	edi: u32,
	/// ESI register
	esi: u32,
	/// EBP register
	ebp: u32,
	/// (pseudo) ESP register
	esp: u32,
	/// EBX register
	ebx: u32,
	/// EDX register
	edx: u32,
	/// ECX register
	ecx: u32,
	/// EAX register
	eax: u32,
	/// status flags
	eflags: u32,
	/// instruction pointer
	eip: u32,
}

extern "C" fn leave_task() -> ! {
	debug!("finish task {}", get_current_taskid());

	do_exit();
}

impl TaskFrame for Task {
	#[cfg(target_arch = "x86_64")]
	fn create_stack_frame(&mut self, func: extern "C" fn()) {
		unsafe {
			let mut stack: *mut u64 = ((*self.stack).top()).as_mut_ptr();

			write_bytes((*self.stack).bottom().as_mut_ptr::<u8>(), 0xCD, STACK_SIZE);

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
			(*state).gs = ((*self.stack).top()).as_u64();

			(*state).rip = (func as *const ()) as u64;
			(*state).rflags = 0x1202u64;

			/* Set the task's stack pointer entry to the stack we have crafted right now. */
			self.last_stack_pointer = VirtAddr(stack as u64);
		}
	}

	#[cfg(target_arch = "x86")]
	fn create_stack_frame(&mut self, func: extern "C" fn()) {
		unsafe {
			let mut stack: *mut u32 = ((*self.stack).top()).as_mut_ptr();

			write_bytes((*self.stack).bottom().as_mut_ptr::<u8>(), 0xCD, STACK_SIZE);

			/* Only marker for debugging purposes, ... */
			*stack = 0xDEADBEEFu32;
			stack = (stack as usize - size_of::<u32>()) as *mut u32;

			/* the first-function-to-be-called's arguments, ... */
			//TODO: add arguments

			/* and the "caller" we shall return to.
			 * This procedure cleans the task after exit. */
			*stack = (leave_task as *const ()) as u32;
			stack = (stack as usize - size_of::<State>()) as *mut u32;

			let state: *mut State = stack as *mut State;
			write_bytes(state, 0x00, 1);

			(*state).esp = (stack as usize + size_of::<State>()) as u32;
			(*state).ebp = (*state).esp + size_of::<u32>() as u32;

			(*state).eip = (func as *const ()) as u32;
			(*state).eflags = 0x1002u32;

			/* Set the task's stack pointer entry to the stack we have crafted right now. */
			self.last_stack_pointer = stack as usize;
		}
	}
}
