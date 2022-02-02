// Copyright (c) 2017-2018 Stefan Lankes, RWTH Aachen University
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

use crate::arch::x86_64::kernel::gdt::set_current_kernel_stack;
use core::arch::asm;

macro_rules! save_context {
	() => {
		concat!(
			r#"
			pushfq
			push rax
			push rcx
			push rdx
			push rbx
			sub  rsp, 8
			push rbp
			push rsi
			push rdi
			push r8
			push r9
			push r10
			push r11
			push r12
			push r13
			push r14
			push r15
			"#,
		)
	};
}

macro_rules! restore_context {
	() => {
		concat!(
			r#"
			pop r15
			pop r14
			pop r13
			pop r12
			pop r11
			pop r10
			pop r9
			pop r8
			pop rdi
			pop rsi
			pop rbp
			add rsp, 8
			pop rbx
			pop rdx
			pop rcx
			pop rax
			popfq
			ret
			"#
		)
	};
}

#[naked]
pub unsafe extern "C" fn switch(_old_stack: *mut usize, _new_stack: usize) {
	// rdi = old_stack => the address to store the old rsp
	// rsi = new_stack => stack pointer of the new task

	asm!(
		save_context!(),
		"rdfsbase rax",
		"rdgsbase rdx",
		"push rax",
		"push rdx",
		// Store the old `rsp` behind `old_stack`
		"mov [rdi], rsp",
		// Set `rsp` to `new_stack`
		"mov rsp, rsi",
		// Set task switched flag
		"mov rax, cr0",
		"or rax, 8",
		"mov cr0, rax",
		// set stack pointer in TSS
		"call {set_stack}",
		"pop r15",
		"wrgsbase r15",
		"pop r15",
		"wrfsbase r15",
		restore_context!(),
		set_stack = sym set_current_kernel_stack,
		options(noreturn)
	);
}
