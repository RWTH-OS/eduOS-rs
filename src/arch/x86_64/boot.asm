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

;;; Based on http://blog.phil-opp.com/rust-os/multiboot-kernel.html
;;; and http://blog.phil-opp.com/rust-os/entering-longmode.html
;;;
;;; The actual boot code of our kernel.

%include 'common.inc'

global start
global gdt64_code_offset
global HEAP_BOTTOM
global HEAP_TOP
global MBINFO

extern long_mode_start
extern kernel_start
extern kernel_end

KERNEL_STACK_SIZE equ 8192
PAGE_SIZE equ 4096

;;; Our main entry point.  Invoked by out boot loader.
section .text
bits 32
start:
        mov esp, stack_top-16              ; Use our temporary stack.
        mov DWORD [MBINFO], ebx            ; store the pointer to the multiboot info

        ; Sanity-check our system.
        call test_multiboot
        call test_cpuid
		; in 64bit mode, we have to enble paging before jumping in 64bit mode
        call test_long_mode

        ;; Turn on paging.
        call setup_page_tables
        call enable_paging

        ;; Install our GDT.
        lgdt [gdt64.pointer]

        ;; Set up our data segment registers.
        mov ax, gdt64.data
        mov ss, ax
        mov ds, ax
        mov es, ax

        ;; To set up our code segment, we need to make a jump, and
        ;; when the jump finishes, we'll be in 64-bit mode.
        jmp gdt64.code:long_mode_start

;;; Boot-time error handler.  Prints `ERR: ` and a code.
;;;
;;; al: Error code.
error:
        ; save error code
        mov ecx, eax
        ; print ERR: to COM1
        mov edx, COM1
        mov eax, 'E'
        out dx, ax
        mov eax, 'R'
        out dx, ax
        out dx, ax
        mov eax, ':'
        out dx, ax
        mov eax, ' '
        out dx, ax
        mov eax, ecx
        add eax, 0x30
        out dx, ax
        mov eax, '\n'
        out dx, ax
        ; halt system
        hlt

;;; Make sure we were loaded by multiboot.
test_multiboot:
        cmp eax, 0x2BADB002     ; Did multiboot put a magic value in eax?
        jne .no_multiboot
        ret
.no_multiboot:
        mov al, "M"
        jmp error

;;; Test for CPUID.  Copied from
;;; http://blog.phil-opp.com/rust-os/entering-longmode.html
;;; which copied from
;;; http://wiki.osdev.org/Setting_Up_Long_Mode#Detection_of_CPUID
test_cpuid:
        pushfd                  ; Store the FLAGS-register.
        pop eax                 ; Restore the A-register.
        mov ecx, eax            ; Set the C-register to the A-register.
        xor eax, 1 << 21        ; Flip the ID-bit, which is bit 21.
        push eax                ; Store the A-register.
        popfd                   ; Restore the FLAGS-register.
        pushfd                  ; Store the FLAGS-register.
        pop eax                 ; Restore the A-register.
        push ecx                ; Store the C-register.
        popfd                   ; Restore the FLAGS-register.
        xor eax, ecx            ; Do a XOR-operation on the A and C.
        jz .no_cpuid            ; The zero flag is set, no CPUID.
        ret                     ; CPUID is available for use.
.no_cpuid:
        mov al, "I"
        jmp error

;;; Test for presence of 64-bit support.  Copied from the same sources as
;;; test_cpuid.
test_long_mode:
        mov eax, 0x80000000     ; Set the A-register to 0x80000000.
        cpuid                   ; CPU identification.
        cmp eax, 0x80000001     ; Compare the A-register with 0x80000001.
        jb .no_long_mode        ; It is less, there is no long mode.
        mov eax, 0x80000001     ; Set the A-register to 0x80000001.
        cpuid                   ; CPU identification.
        ;; Test if the LM-bit, which is bit 29, is set in the D-register.
        test edx, 1 << 29
        jz .no_long_mode        ; They aren't, there is no long mode.
        ret
.no_long_mode:
        mov al, "L"
        jmp error

setup_page_tables:
        ; map multiboot info 1:1
        push edi
        mov eax, DWORD [MBINFO]   ; map multiboot info
        and eax, 0xFFFFF000       ; page align lower half
        mov edi, eax
        shr edi, 9                ; (edi >> 12) * 8 (index for boot_pgt)
        add edi, boot_pgt
        or eax, 0x101             ; set present and global bits
        mov DWORD [edi], eax
        pop edi

        ; map kernel 1:1
        push eax
        push edi
        push ecx
        mov edi, kernel_start
        shr edi, 18                ; (edi >> 21) * 8 (index for boot_pgd)
        add edi, boot_pgd
        mov eax, kernel_start
        or eax, 0x1A7  ; if we want to allow user access 0x1A7 otherwise 0x1A3
        xor ecx, ecx
Lmap:
        mov DWORD [edi], eax
        add eax, 0x200000
        add ecx, 0x200000
        add edi, 8
        ; note: the whole code segement has to fit in the first pgd
        cmp ecx, 0x4000000   ; reserve 64 MB
        jl Lmap
        pop ecx
        pop edi
        pop eax

        ret

;;; Turn on paging.
enable_paging:
        ;; Load P4 into cr3.
        mov eax, boot_pml4
        mov cr3, eax

        ;; Enable Physical Address Extension in cr4.
        mov eax, cr4
        or eax, 0x20
        mov cr4, eax

        ;; Set the long mode bit in the EFER MSR.
        mov ecx, 0xC0000080
        rdmsr
        or eax, 0x100
        wrmsr

        ;; Turn on paging in cr0.
        mov eax, cr0
        or eax, 0x80010000
        mov cr0, eax
        ret

bits 64
global __replace_boot_stack
__replace_boot_stack:
		; rdi = 1st argument = desination address
		; set rsp to the new stack
		sub rsp, stack_bottom
		add rsp, rdi

		; we elimante frame pointers => no recalculation of rbp required
		;sub rbp, stack_bottom
		;add rbp, rdi

		; copy boot stack to the new one
		cld
		mov rcx, KERNEL_STACK_SIZE
		mov rsi, stack_bottom
		rep movsb

		ret

section .data

; Bootstrap page tables are used during the initialization.
align 4096
boot_pml4:
	DQ boot_pdpt + 0x107 ; PG_PRESENT | PG_GLOBAL | PG_RW | PG_USER
	times 510 DQ 0       ; PAGE_MAP_ENTRIES - 2
	DQ boot_pml4 + 0x303 ; PG_PRESENT | PG_GLOBAL | PG_RW | PG_SELF (self-reference)
boot_pdpt:
	DQ boot_pgd + 0x107  ; PG_PRESENT | PG_GLOBAL | PG_RW | PG_USER
	times 510 DQ 0       ; PAGE_MAP_ENTRIES - 2
	DQ boot_pml4 + 0x303 ; PG_PRESENT | PG_GLOBAL | PG_RW | PG_SELF (self-reference)
boot_pgd:
	DQ boot_pgt + 0x107  ; PG_PRESENT | PG_GLOBAL | PG_RW | PG_USER
	times 510 DQ 0       ; PAGE_MAP_ENTRIES - 2
	DQ boot_pml4 + 0x303 ; PG_PRESENT | PG_GLOBAL | PG_RW | PG_SELF (self-reference)
boot_pgt:
	times 512 DQ 0

;;; Our kernel stack.  We want to make this large enough so that we don't
;;; need to worry about overflowing it until we figure out how to set up
;;; a guard page and print errors on page faults.
align PAGE_SIZE
stack_bottom:
		TIMES KERNEL_STACK_SIZE DB 0x00
stack_top:

section .rodata
;;; Global Description Table.  Used to set segmentation to the restricted
;;; values needed for 64-bit mode.
gdt64:
    dq 0                                                ; Mandatory 0.
.code: equ $ - gdt64
    dq (1<<44) | (1<<47) | (1<<41) | (1<<43) | (1<<53)  ; Code segment.
.data: equ $ - gdt64
    dq (1<<44) | (1<<47) | (1<<41)                      ; Data segment.
.pointer:
    dw $ - gdt64 - 1
    dq gdt64

;;; Export selectors so Rust can access them.
gdt64_code_offset:
    dw gdt64.code

section .data
MBINFO DD 0
