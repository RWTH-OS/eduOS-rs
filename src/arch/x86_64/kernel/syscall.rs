// Copyright (c) 2017-2018 Stefan Lankes, RWTH Aachen University
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

#[no_mangle]
#[naked]
pub unsafe extern "C" fn syscall_handler() {
	asm!(
		// save context, see x86_64 ABI
		"push %rcx\n\t\
		push %rdx\n\t\
		push %rsi\n\t\
		push %rdi\n\t\
		push %r8\n\t\
		push %r9\n\t\
		push %r10\n\t\
		push %r11\n\t\
		// save ds/es and set to kernel data descriptor \n\t\
		mov %ds, %rcx\n\t\
		push %rcx\n\t\
		mov %es, %rcx\n\t\
		push %rcx\n\t\
		mov $$0x10, %rcx\n\t\
		mov %rcx, %ds\n\t\
		mov %rcx, %es\n\t\
		// switch to kernel stack\n\t
		swapgs\n\t
		mov %rsp, %rcx\n\t
		rdgsbase %rsp\n\t
		push %rcx
		// copy 4th argument to rcx to adhere x86_64 ABI \n\t\
		mov %r10, %rcx\n\t\
		sti\n\t\
		call *SYSHANDLER_TABLE(,%rax,8)\n\t
		// restore context, see x86_64 ABI \n\t\
		cli\n\t\
		// switch to user stack\n\t
		pop %rcx\n\t
		mov %rcx, %rsp\n\t
		swapgs\n\t\
		// restore context
		pop %rcx\n\t\
		mov %rcx, %es\n\t\
	    pop %rcx\n\t\
		mov %rcx, %ds\n\t\
		pop %r11\n\t\
		pop %r10\n\t\
		pop %r9\n\t\
		pop %r8\n\t\
		pop %rdi\n\t\
		pop %rsi\n\t\
		pop %rdx\n\t\
		pop %rcx\n\t\
		sysretq" :::: "volatile");
}
