//! Architecture dependent interface to initialize a task

use crate::scheduler::do_exit;
use crate::scheduler::task::*;
use core::mem::size_of;

#[repr(C, packed)]
struct State {
	/// program status register
	spsr_el1: u64,
	/// X0 register
	x0: u64,
	/// X1 register
	x1: u64,
	/// X2 register
	x2: u64,
	/// X3 register
	x3: u64,
	/// X4 register
	x4: u64,
	/// X5 register
	x5: u64,
	/// X6 register
	x6: u64,
	/// X7 register
	x7: u64,
	/// X8 register
	x8: u64,
	/// X9 register
	x9: u64,
	/// X10 register
	x10: u64,
	/// X11 register
	x11: u64,
	/// X12 register
	x12: u64,
	/// X13 register
	x13: u64,
	/// X14 register
	x14: u64,
	/// X15 register
	x15: u64,
	/// X16 register
	x16: u64,
	/// X17 register
	x17: u64,
	/// X18 register
	x18: u64,
	/// X19 register
	x19: u64,
	/// X20 register
	x20: u64,
	/// X21 register
	x21: u64,
	/// X22 register
	x22: u64,
	/// X23 register
	x23: u64,
	/// X24 register
	x24: u64,
	/// X25 register
	x25: u64,
	/// X26 register
	x26: u64,
	/// X27 register
	x27: u64,
	/// X28 register
	x28: u64,
	/// X29 register (frame pointer)
	x29: u64,
	/// X30 register (called link register)
	x30: u64,
}

extern "C" fn entry_point(func: extern "C" fn()) -> ! {
	func();
	do_exit();
}

impl TaskFrame for Task {
	fn create_stack_frame(&mut self, func: extern "C" fn()) {
		// note, stack is already zeroed

		unsafe {
			let mut stack: *mut u64 = ((*self.stack).top()).as_mut_ptr();

			/* Only marker for debugging purposes, ... */
			*stack = 0xDEADBEEFu64;
			stack = (stack as usize - size_of::<u64>()) as *mut u64;
			*stack = 0xDEADBEEFu64;
			stack = (stack as usize - size_of::<u64>()) as *mut u64;
			stack = (stack as usize - size_of::<State>()) as *mut u64;

			let state: *mut State = stack as *mut State;
			(*state).x0 = (func as *const ()) as u64;
			(*state).x29 = 0; // frame pointer
			(*state).x30 = (entry_point as *const ()) as u64;
			/* Zero the condition flags. */
			(*state).spsr_el1 = 0x3e5;

			/* Set the task's stack pointer entry to the stack we have crafted right now. */
			self.last_stack_pointer = stack as usize;
		}
	}
}
