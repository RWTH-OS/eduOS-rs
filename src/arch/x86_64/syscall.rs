// Copyright (c) 2017-2018 Stefan Lankes, RWTH Aachen University
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

use crate::syscall::SYSHANDLER_TABLE;

#[no_mangle]
#[naked]
pub unsafe extern "C" fn syscall_handler() {
	asm!(
		// save context, see x86_64 ABI
		"push rcx",
		"push rdx",
		"push rsi",
		"push rdi",
		"push r8",
		"push r9",
		"push r10",
		"push r11",
		// save ds/es and set to kernel data descriptor \n\t\
		"mov rcx, ds",
		"push rcx",
		"mov rcx, es",
		"push rcx",
		"mov rcx, {kernel_ds}",
		"mov ds, rcx",
		"mov es, rcx",
		// copy 4th argument to rcx to adhere x86_64 ABI \n\t\
		"mov rcx, r10",
		"sti",
		"call [{sys_handler}+8*rax]",
		// restore context, see x86_64 ABI \n\t\
		"cli",
		"pop rcx",
		"mov es, rcx",
		"pop rcx",
		"mov ds, rcx",
		"pop r11",
		"pop r10",
		"pop r9",
		"pop r8",
		"pop rdi",
		"pop rsi",
		"pop rdx",
		"pop rcx",
		"sysretq",
		sys_handler = sym SYSHANDLER_TABLE,
		kernel_ds = const 0x10,
		options(noreturn));
}
