// Entry point for the RISC-V 64 kernel.
//
// QEMU's `virt` machine (with `-bios none`) loads the kernel ELF at its link
// address and starts every hart in M-mode with a0 = hartid and a1 = pointer to
// the device tree.

.section .text._start

_start:
	// Only the boot hart (hartid 0) proceeds; park all other harts.
	bnez	a0, 1f

	// Set up the global pointer. Relaxation has to be disabled here, otherwise
	// the assembler would relax `la gp, ...` against gp itself.
	.option push
	.option norelax
	la		gp, __global_pointer$
	.option pop

	// Set up the boot stack (the stack grows downwards).
	la		sp, __boot_core_stack_end_exclusive

	// Jump to Rust code.
	j		{start_rust}

	// Infinitely wait for interrupts (aka "park the hart").
1:	wfi
	j		1b

.size	_start, . - _start
.type	_start, function
.global	_start
