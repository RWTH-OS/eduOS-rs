// Copyright (c) 2020 Stefan Lankes, RWTH Aachen University
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

.section .text
.global __switch
.extern set_current_kernel_stack

.align 16
__switch:
	// rdi = old_stack => the address to store the old rsp
	// rsi = new_stack => stack pointer of the new task

	pushfq
	push %rax
	push %rcx
	push %rdx
	push %rbx
	sub  $8, %rsp	// ignore rsp
	push %rbp
	push %rsi
	push %rdi
	push %r8
	push %r9
	push %r10
	push %r11
	push %r12
	push %r13
	push %r14
	push %r15
    // push fs registers
	rdfsbaseq %rax
	push %rax
	// push gs registers
	rdgsbaseq %rax
	push %rax
	// store the old stack pointer in the dereferenced first parameter\n\t\
	// and load the new stack pointer in the second parameter.\n\t\
	mov %rsp, (%rdi)
	mov %rsi, %rsp
	// Set task switched flag
	mov %cr0, %rax
	or $8, %rax
	mov %rax, %cr0
	// set stack pointer in TSS
	call set_current_kernel_stack
	// restore context
	pop %rax
	wrgsbaseq %rax
	pop %rax
	wrfsbaseq %rax
	pop %r15
	pop %r14
	pop %r13
	pop %r12
	pop %r11
	pop %r10
	pop %r9
	pop %r8
	pop %rdi
	pop %rsi
	pop %rbp
	add $8, %rsp
	pop %rbx
	pop %rdx
	pop %rcx
	pop %rax
	popfq
	ret
