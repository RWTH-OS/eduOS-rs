; Copyright (c) 2018 Stefan Lankes, RWTH Aachen University
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

KERNEL_STACK_SIZE equ 8192
PAGE_SIZE equ 4096

section .text

global rust_start
rust_start:
	mov rsp, stack_top-16 ; Use our temporary stack.

	extern rust_main
	call rust_main

	; halt system
	extern shutdown
L1:
	call shutdown
	jmp L1

;;; Our kernel stack.  We want to make this large enough so that we don't
;;; need to worry about overflowing it until we figure out how to set up
;;; a guard page and print errors on page faults.
align PAGE_SIZE
stack_bottom:
	resb KERNEL_STACK_SIZE
stack_top:
