// Adapted from https://github.com/rust-embedded/rust-raspberrypi-OS-tutorials/blob/master/02_runtime_init/src/_arch/aarch64/cpu/boot.s

.equ _core_id_mask, 0xff

.section .text

_start:
	// Only proceed on the boot core. Park it otherwise.
	mrs	x1, mpidr_el1
	and	x1, x1, _core_id_mask
	mov	x2, xzr  // Assume CPU 0 is responsible for booting
	cmp	x1, x2
	b.ne	1f

	// If execution reaches here, it is the boot core. Now, prepare the jump to Rust code.

	// This loads the physical address of the stack end. For details see
	// https://github.com/rust-embedded/rust-raspberrypi-OS-tutorials/blob/master/16_virtual_mem_part4_higher_half_kernel/src/bsp/raspberrypi/link.ld
	adrp	x4, __boot_core_stack_end_exclusive
	add		x4, x4, #:lo12:__boot_core_stack_end_exclusive
	mov		sp, x4
	
	// Jump to Rust code.
	b	{start_rust}

	// Infinitely wait for events (aka "park the core").
1:	wfe
	b	1b

.size	_start, . - _start
.type	_start, function
.global	_start