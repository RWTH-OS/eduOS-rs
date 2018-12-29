/* Copyright (c) 2017-2018 Stefan Lankes, RWTH Aachen University
 *
 * MIT License
 *
 * Permission is hereby granted, free of charge, to any person obtaining
 * a copy of this software and associated documentation files (the
 * "Software"), to deal in the Software without restriction, including
 * without limitation the rights to use, copy, modify, merge, publish,
 * distribute, sublicense, and/or sell copies of the Software, and to
 * permit persons to whom the Software is furnished to do so, subject to
 * the following conditions:
 *
 * The above copyright notice and this permission notice shall be
 * included in all copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND,
 * EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF
 * MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND
 * NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE
 * LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION
 * OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION
 * WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
 */

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
		// copy 4th argument to rcx to adhere x86_64 ABI \n\t\
		mov %r10, %rcx\n\t\
		sti\n\t\
		call *SYSHANDLER_TABLE(,%rax,8)\n\t
		// restore context, see x86_64 ABI \n\t\
		cli\n\t\
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
