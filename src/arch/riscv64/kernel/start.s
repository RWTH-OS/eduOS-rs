// Entry point for the RISC-V 64 kernel.
//
// QEMU's `virt` machine (with `-bios none`) loads the kernel ELF at its link
// address and starts every hart in M-mode with a0 = hartid and a1 = pointer to
// the device tree.
//
// The kernel itself runs in S-mode. Without a firmware (no OpenSBI) we use a
// tiny M-mode shim that configures the machine once and then drops to S-mode
// with `mret`. After that, M-mode is never entered again: all traps are
// delegated to S-mode and the timer is programmed via the Sstc extension
// (`stimecmp`).

.section .text._start

_start:
	// Only the boot hart (hartid 0) proceeds; park all other harts.
	bnez	a0, 2f

	// Set up the global pointer. Relaxation has to be disabled here, otherwise
	// the assembler would relax `la gp, ...` against gp itself.
	.option push
	.option norelax
	la		gp, __global_pointer$
	.option pop

	// Set up the boot stack (the stack grows downwards).
	la		sp, __boot_core_stack_end_exclusive

	// ---- M-mode shim -------------------------------------------------------

	// Any trap that still reaches M-mode is a bug: terminate QEMU with an
	// error code instead of hanging.
	la		t0, m_trap
	csrw	mtvec, t0

	// PMP: without at least one entry, S- and U-mode cannot access any memory.
	// Entry 0 = TOR (top of range) up to the end of the address space, RWX.
	li		t0, -1
	csrw	pmpaddr0, t0
	li		t0, 0x0f				// A = TOR, X, W, R
	csrw	pmpcfg0, t0

	// Delegate all exceptions and the supervisor interrupts to S-mode.
	li		t0, 0xffff
	csrw	medeleg, t0
	li		t0, (1 << 1) | (1 << 5) | (1 << 9)	// SSIP, STIP, SEIP
	csrw	mideleg, t0

	// Sstc: allow S-mode to use `stimecmp` (menvcfg.STCE, bit 63) and to read
	// the `time` CSR (mcounteren.TM, bit 1).
	li		t0, 1
	slli	t0, t0, 63
	csrs	menvcfg, t0
	csrsi	mcounteren, 2

	// No address translation yet.
	csrw	satp, zero

	// mret to S-mode: mstatus.MPP = 01 (S), mepc = start_rust.
	li		t0, 3 << 11
	csrc	mstatus, t0
	li		t0, 1 << 11
	csrs	mstatus, t0
	la		t0, {start_rust}
	csrw	mepc, t0
	mret

	// Infinitely wait for interrupts (aka "park the hart").
2:	wfi
	j		2b

.size	_start, . - _start
.type	_start, function
.global	_start

// M-mode trap handler of the shim: exit QEMU via the SiFive test finisher
// with error code 3 (0x3333 | 3 << 16).
.balign 4
m_trap:
	li		t0, 0x00100000
	li		t1, 0x00033333
	sw		t1, 0(t0)
3:	j		3b
