; Copyright (c) 2017 Stefan Lankes, RWTH Aachen University
;
; MIT License
;
; Permission is hereby granted, free of charge, to any person obtaining
; a copy of this software and associated documentation files (the
; "Software"), to deal in the Software without restriction, including
; without limitation the rights to use, copy, modify, merge, publish,
; distribute, sublicense, and/or sell copies of the Software, and to
; permit persons to whom the Software is furnished to do so, subject to
; the following conditions:
;
; The above copyright notice and this permission notice shall be
; included in all copies or substantial portions of the Software.
;
; THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND,
; EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF
; MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND
; NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE
; LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION
; OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION
; WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.

section .text
bits 64

extern sys_write
extern sys_exit

global syscall_handler
syscall_handler:
	; save context, see x86_64 ABI
	push rcx
	push rdx
	push rsi
	push rdi
	push r8
	push r9
	push r10
	push r11

	; save ds/es and set to kernel data descriptor
	mov rcx, ds
	push rcx
	mov rcx, es
	push rcx
	mov rcx, 0x10
	mov ds, rcx
	mov es, rcx

	; copy 4th argument to rcx to adhere x86_64 ABI
	mov rcx, r10
	sti

	call [sys_handlers+rax*8]

	; restore context, see x86_64 ABI
	cli
	pop rcx
	mov es, rcx
    pop rcx
	mov ds, rcx

	pop r11
	pop r10
	pop r9
	pop r8
	pop rdi
	pop rsi
	pop rdx
	pop rcx

	o64 sysret

section .rodata

; array of function pointers, which handles the system calls
sys_handlers:
	dq sys_exit
	dq sys_write
	dq 0 ; signalize the end of the array
