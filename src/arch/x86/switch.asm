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

%ifidn __OUTPUT_FORMAT__, elf64
section .text
bits 64

align 8
rollback:
	ret

global switch
align 8
switch:
	; rdi => the address to store the old rsp
	; rsi => stack pointer of the new task

	; create on the stack a pseudo interrupt
	; afterwards, we switch to the task with iret
	push QWORD 0x10				; SS
	push rsp					; RSP
	add QWORD [rsp], 0x08		; => value of rsp before the creation of a pseudo interrupt
	pushfq						; RFLAGS
	push QWORD 0x08				; CS
	push QWORD rollback			; RIP
	push QWORD 0x00edbabe		; Error code
	push QWORD 0x00				; Interrupt number

	; save context
	push rax
	push rcx
	push rdx
	push rbx
	push QWORD [rsp+9*8]
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

	mov QWORD [rdi], rsp				; store old rsp
	mov rsp, rsi

	; restore context
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

	add rsp, 16
	iretq
%elifidn __OUTPUT_FORMAT__, elf32
section .text
bits 32

align 4
rollback:
	ret

; Create a pseudo interrupt on top of the stack.
; Afterwards, we switch to the task with iret.
; We already are in kernel space => no pushing of SS required.
global switch
align 4
switch:
    pushf                       ; push controll register
    push DWORD 0x8              ; CS
    push DWORD rollback         ; EIP
    push DWORD 0x0              ; Interrupt number
    push DWORD 0x00edbabe       ; Error code
    pusha                       ; push all general purpose registers...
    push 0x10                   ; kernel data segment (for ES)
    push 0x10                   ; kernel data segment (for DS)

	; 1st argument => the address to store the old rsp
    ; 2nd argument => stack pointer of the new task
	mov edi, DWORD [esp+16*4]
    mov esi, DWORD [esp+17*4]

	mov DWORD [edi], esp        ; store old esp
	mov esp, esi

    pop ds
    pop es
    popa
    add esp, 8
    iret
%endif
