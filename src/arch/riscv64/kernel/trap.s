// RISC-V machine-mode trap vector.
//
// The kernel runs in M-mode (QEMU `virt` with `-bios none`), so all traps are
// delivered here through `mtvec` (direct mode). On entry the full integer
// context is saved onto the current (task) stack, the Rust dispatcher
// `handle_trap(frame)` is called, and finally the context is restored and the
// trap returns with `mret`.
//
// Frame layout (30 general purpose registers + mepc + mstatus = 256 bytes,
// 16-byte aligned). x0 (zero) and x2 (sp, managed by the frame) are not saved.

.equ FRAME_SIZE, 256

.section .text
.balign 4
.global trap_entry
trap_entry:
	addi	sp, sp, -FRAME_SIZE
	sd		x1, 0*8(sp)    // ra
	sd		x3, 1*8(sp)    // gp
	sd		x4, 2*8(sp)    // tp
	sd		x5, 3*8(sp)    // t0
	sd		x6, 4*8(sp)    // t1
	sd		x7, 5*8(sp)    // t2
	sd		x8, 6*8(sp)    // s0
	sd		x9, 7*8(sp)    // s1
	sd		x10, 8*8(sp)   // a0
	sd		x11, 9*8(sp)   // a1
	sd		x12, 10*8(sp)  // a2
	sd		x13, 11*8(sp)  // a3
	sd		x14, 12*8(sp)  // a4
	sd		x15, 13*8(sp)  // a5
	sd		x16, 14*8(sp)  // a6
	sd		x17, 15*8(sp)  // a7
	sd		x18, 16*8(sp)  // s2
	sd		x19, 17*8(sp)  // s3
	sd		x20, 18*8(sp)  // s4
	sd		x21, 19*8(sp)  // s5
	sd		x22, 20*8(sp)  // s6
	sd		x23, 21*8(sp)  // s7
	sd		x24, 22*8(sp)  // s8
	sd		x25, 23*8(sp)  // s9
	sd		x26, 24*8(sp)  // s10
	sd		x27, 25*8(sp)  // s11
	sd		x28, 26*8(sp)  // t3
	sd		x29, 27*8(sp)  // t4
	sd		x30, 28*8(sp)  // t5
	sd		x31, 29*8(sp)  // t6
	csrr	t0, mepc
	sd		t0, 30*8(sp)
	csrr	t0, mstatus
	sd		t0, 31*8(sp)

	// Call the Rust dispatcher with a pointer to the saved frame.
	mv		a0, sp
	call	{handle_trap}

	// Restore mepc and mstatus first (using t0 as scratch, restored below).
	ld		t0, 31*8(sp)
	csrw	mstatus, t0
	ld		t0, 30*8(sp)
	csrw	mepc, t0

	ld		x1, 0*8(sp)
	ld		x3, 1*8(sp)
	ld		x4, 2*8(sp)
	ld		x5, 3*8(sp)
	ld		x6, 4*8(sp)
	ld		x7, 5*8(sp)
	ld		x8, 6*8(sp)
	ld		x9, 7*8(sp)
	ld		x10, 8*8(sp)
	ld		x11, 9*8(sp)
	ld		x12, 10*8(sp)
	ld		x13, 11*8(sp)
	ld		x14, 12*8(sp)
	ld		x15, 13*8(sp)
	ld		x16, 14*8(sp)
	ld		x17, 15*8(sp)
	ld		x18, 16*8(sp)
	ld		x19, 17*8(sp)
	ld		x20, 18*8(sp)
	ld		x21, 19*8(sp)
	ld		x22, 20*8(sp)
	ld		x23, 21*8(sp)
	ld		x24, 22*8(sp)
	ld		x25, 23*8(sp)
	ld		x26, 24*8(sp)
	ld		x27, 25*8(sp)
	ld		x28, 26*8(sp)
	ld		x29, 27*8(sp)
	ld		x30, 28*8(sp)
	ld		x31, 29*8(sp)
	addi	sp, sp, FRAME_SIZE
	mret

.size	trap_entry, . - trap_entry
.type	trap_entry, function
