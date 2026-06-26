// AArch64 exception vector table for EL1.
//
// Each of the 16 vector entries saves the full integer context onto the current
// (SP_EL1) stack and branches to a common trampoline that calls the Rust
// dispatcher `handle_exception(frame, vector_index)` and finally restores the
// context and returns from the exception with `eret`.
//
// Frame layout (matches `irq::TrapFrame`):
//   offset 0   : x0..x30 (31 * 8 bytes)
//   offset 248 : ELR_EL1
//   offset 256 : SPSR_EL1
//   offset 264 : padding (keeps the frame 16-byte aligned, total 272 bytes)

.macro SAVE_CONTEXT
	sub		sp, sp, #272
	stp		x0, x1, [sp, #16 * 0]
	stp		x2, x3, [sp, #16 * 1]
	stp		x4, x5, [sp, #16 * 2]
	stp		x6, x7, [sp, #16 * 3]
	stp		x8, x9, [sp, #16 * 4]
	stp		x10, x11, [sp, #16 * 5]
	stp		x12, x13, [sp, #16 * 6]
	stp		x14, x15, [sp, #16 * 7]
	stp		x16, x17, [sp, #16 * 8]
	stp		x18, x19, [sp, #16 * 9]
	stp		x20, x21, [sp, #16 * 10]
	stp		x22, x23, [sp, #16 * 11]
	stp		x24, x25, [sp, #16 * 12]
	stp		x26, x27, [sp, #16 * 13]
	stp		x28, x29, [sp, #16 * 14]
	mrs		x9, elr_el1
	mrs		x10, spsr_el1
	stp		x30, x9, [sp, #16 * 15]
	str		x10, [sp, #16 * 16]
.endm

.macro RESTORE_CONTEXT
	ldr		x10, [sp, #16 * 16]
	ldp		x30, x9, [sp, #16 * 15]
	msr		spsr_el1, x10
	msr		elr_el1, x9
	ldp		x0, x1, [sp, #16 * 0]
	ldp		x2, x3, [sp, #16 * 1]
	ldp		x4, x5, [sp, #16 * 2]
	ldp		x6, x7, [sp, #16 * 3]
	ldp		x8, x9, [sp, #16 * 4]
	ldp		x10, x11, [sp, #16 * 5]
	ldp		x12, x13, [sp, #16 * 6]
	ldp		x14, x15, [sp, #16 * 7]
	ldp		x16, x17, [sp, #16 * 8]
	ldp		x18, x19, [sp, #16 * 9]
	ldp		x20, x21, [sp, #16 * 10]
	ldp		x22, x23, [sp, #16 * 11]
	ldp		x24, x25, [sp, #16 * 12]
	ldp		x26, x27, [sp, #16 * 13]
	ldp		x28, x29, [sp, #16 * 14]
	add		sp, sp, #272
	eret
.endm

.macro VECTOR_ENTRY index
.balign 0x80
	SAVE_CONTEXT
	mov		x0, sp
	mov		x1, #\index
	b		__exception_common
.endm

.section .text
.balign 0x800
.global vector_table_el1
vector_table_el1:
	VECTOR_ENTRY 0   // Current EL with SP0:  Synchronous
	VECTOR_ENTRY 1   // Current EL with SP0:  IRQ
	VECTOR_ENTRY 2   // Current EL with SP0:  FIQ
	VECTOR_ENTRY 3   // Current EL with SP0:  SError
	VECTOR_ENTRY 4   // Current EL with SPx:  Synchronous
	VECTOR_ENTRY 5   // Current EL with SPx:  IRQ
	VECTOR_ENTRY 6   // Current EL with SPx:  FIQ
	VECTOR_ENTRY 7   // Current EL with SPx:  SError
	VECTOR_ENTRY 8   // Lower EL using AArch64: Synchronous
	VECTOR_ENTRY 9   // Lower EL using AArch64: IRQ
	VECTOR_ENTRY 10  // Lower EL using AArch64: FIQ
	VECTOR_ENTRY 11  // Lower EL using AArch64: SError
	VECTOR_ENTRY 12  // Lower EL using AArch32: Synchronous
	VECTOR_ENTRY 13  // Lower EL using AArch32: IRQ
	VECTOR_ENTRY 14  // Lower EL using AArch32: FIQ
	VECTOR_ENTRY 15  // Lower EL using AArch32: SError

__exception_common:
	bl		{handle_exception}
	RESTORE_CONTEXT
