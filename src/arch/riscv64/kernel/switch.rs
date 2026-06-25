use core::arch::naked_asm;

/// # Safety
///
/// Only the scheduler itself should call this function to switch the
/// context. `old_stack` is a pointer, where the address to the old
/// stack is stored. `new_stack` provides the stack pointer of the
/// next task.
///
/// On RISC-V only the callee-saved registers (`ra` and `s0`-`s11`) have to be
/// preserved across the cooperative switch. In addition `a0` is saved and
/// restored so that a freshly created task can receive its entry function as
/// first argument (see `task::create_stack_frame`).
#[unsafe(naked)]
pub(crate) unsafe extern "C" fn switch(_old_stack: *mut usize, _new_stack: usize) {
	naked_asm!(
		// Reserve space for the saved context on the current stack.
		"addi sp, sp, -14*8",
		"sd ra, 0*8(sp)",
		"sd s0, 1*8(sp)",
		"sd s1, 2*8(sp)",
		"sd s2, 3*8(sp)",
		"sd s3, 4*8(sp)",
		"sd s4, 5*8(sp)",
		"sd s5, 6*8(sp)",
		"sd s6, 7*8(sp)",
		"sd s7, 8*8(sp)",
		"sd s8, 9*8(sp)",
		"sd s9, 10*8(sp)",
		"sd s10, 11*8(sp)",
		"sd s11, 12*8(sp)",
		"sd a0, 13*8(sp)",
		// Store the old stack pointer behind `old_stack` (a0).
		"sd sp, 0(a0)",
		// Switch to the new stack (a1).
		"mv sp, a1",
		// Restore the context of the next task.
		"ld ra, 0*8(sp)",
		"ld s0, 1*8(sp)",
		"ld s1, 2*8(sp)",
		"ld s2, 3*8(sp)",
		"ld s3, 4*8(sp)",
		"ld s4, 5*8(sp)",
		"ld s5, 6*8(sp)",
		"ld s6, 7*8(sp)",
		"ld s7, 8*8(sp)",
		"ld s8, 9*8(sp)",
		"ld s9, 10*8(sp)",
		"ld s10, 11*8(sp)",
		"ld s11, 12*8(sp)",
		"ld a0, 13*8(sp)",
		"addi sp, sp, 14*8",
		"ret",
	);
}
