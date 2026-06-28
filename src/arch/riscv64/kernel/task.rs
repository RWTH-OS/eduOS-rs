//! Architecture dependent interface to initialize a task

use crate::arch::riscv64::kernel::irq;
use crate::scheduler::do_exit;
use crate::scheduler::task::*;
use core::mem::size_of;

/// The saved context of a task. The layout has to match the load/store
/// sequence in `switch::switch`.
#[repr(C, packed)]
struct State {
	/// return address register
	ra: usize,
	/// callee-saved register s0 (frame pointer)
	s0: usize,
	/// callee-saved register s1
	s1: usize,
	/// callee-saved register s2
	s2: usize,
	/// callee-saved register s3
	s3: usize,
	/// callee-saved register s4
	s4: usize,
	/// callee-saved register s5
	s5: usize,
	/// callee-saved register s6
	s6: usize,
	/// callee-saved register s7
	s7: usize,
	/// callee-saved register s8
	s8: usize,
	/// callee-saved register s9
	s9: usize,
	/// callee-saved register s10
	s10: usize,
	/// callee-saved register s11
	s11: usize,
	/// argument register a0, holds the entry function of a new task
	a0: usize,
}

extern "C" fn entry_point(func: extern "C" fn()) -> ! {
	// A new task is entered through the cooperative `switch` (via `ret`), which
	// does not restore `mstatus`. As the very first task is started from within
	// the timer trap handler, interrupts are still globally masked here. Enable
	// them so that this task can be preempted as well.
	irq::irq_enable();

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

			// Reserve space for the saved context and keep the stack pointer
			// 16-byte aligned as required by the RISC-V calling convention.
			stack = (stack as usize - size_of::<State>()) as *mut u64;
			stack = ((stack as usize) & !0xf) as *mut u64;

			let state: *mut State = stack as *mut State;
			// `switch` restores `ra` and jumps to it, so the new task starts in
			// `entry_point` with `func` provided in `a0`.
			(*state).ra = (entry_point as *const ()) as usize;
			(*state).a0 = (func as *const ()) as usize;

			/* Set the task's stack pointer entry to the stack we have crafted right now. */
			self.last_stack_pointer = stack as usize;
		}
	}
}
