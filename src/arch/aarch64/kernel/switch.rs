use core::arch::naked_asm;

/// # Safety
///
/// Only the scheduler itself should call this function to switch the
/// context. `old_stack` is a pointer, where the address to the old
/// stack is stored. `new_stack` provides the stack pointer of the
/// next task.
#[unsafe(naked)]
pub(crate) unsafe extern "C" fn switch(_old_stack: *mut usize, _new_stack: usize) {
	naked_asm!(
		"stp x29, x30, [sp, #-16]!",
		"stp x27, x28, [sp, #-16]!",
		"stp x25, x26, [sp, #-16]!",
		"stp x23, x24, [sp, #-16]!",
		"stp x21, x22, [sp, #-16]!",
		"stp x19, x20, [sp, #-16]!",
		"stp x17, x18, [sp, #-16]!",
		"stp x15, x16, [sp, #-16]!",
		"stp x13, x14, [sp, #-16]!",
		"stp x11, x12, [sp, #-16]!",
		"stp x9, x10, [sp, #-16]!",
		"stp x7, x8, [sp, #-16]!",
		"stp x5, x6, [sp, #-16]!",
		"stp x3, x4, [sp, #-16]!",
		"stp x1, x2, [sp, #-16]!",
		"mrs x22, spsr_el1",
		"stp x22, x0, [sp, #-16]!",
		// Store the old `sp` behind `old_stack`
		"mov x2, sp",
		"str x2, [x0]",
		// Set `sp` to `new_stack`
		"mov sp, x1",
		"ldp x22, x0, [sp], #16",
		"msr spsr_el1, x22",
		"ldp x1, x2, [sp], #16",
		"ldp x3, x4, [sp], #16",
		"ldp x5, x6, [sp], #16",
		"ldp x7, x8, [sp], #16",
		"ldp x9, x10, [sp], #16",
		"ldp x11, x12, [sp], #16",
		"ldp x13, x14, [sp], #16",
		"ldp x15, x16, [sp], #16",
		"ldp x17, x18, [sp], #16",
		"ldp x19, x20, [sp], #16",
		"ldp x21, x22, [sp], #16",
		"ldp x23, x24, [sp], #16",
		"ldp x25, x26, [sp], #16",
		"ldp x27, x28, [sp], #16",
		"ldp x29, x30, [sp], #16",
		"ret",
	);
}
