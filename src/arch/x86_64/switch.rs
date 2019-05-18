// Copyright (c) 2017-2018 Stefan Lankes, RWTH Aachen University
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

#[inline(never)]
#[naked]
pub extern "C" fn switch(_old_stack: *mut usize, _new_stack: usize) {
	// rdi = old_stack => the address to store the old rsp
	// rsi = new_stack => stack pointer of the new task

	unsafe {
		asm!(
			// store context
			"pushfq\n\t\
			push %rax\n\t\
			push %rcx\n\t\
			push %rdx\n\t\
			push %rbx\n\t\
			sub  $$8, %rsp	// ignore rsp\n\t\
			push %rbp\n\t\
			push %rsi\n\t\
			push %rdi\n\t\
			push %r8\n\t\
			push %r9\n\t\
			push %r10\n\t\
			push %r11\n\t\
			push %r12\n\t\
			push %r13\n\t\
			push %r14\n\t\
			push %r15\n\t\
			mov %rsp, (%rdi)\n\t\
			mov %rsi, %rsp\n\t\
			// restore context \n\t\
			pop %r15\n\t\
			pop %r14\n\t\
			pop %r13\n\t\
			pop %r12\n\t\
			pop %r11\n\t\
			pop %r10\n\t\
			pop %r9\n\t\
			pop %r8\n\t\
			pop %rdi\n\t\
			pop %rsi\n\t\
			pop %rbp\n\t\
			add $$8, %rsp\n\t\
			pop %rbx\n\t\
			pop %rdx\n\t\
			pop %rcx\n\t\
			pop %rax\n\t\
			popfq" :::: "volatile"
		);
	}
}
