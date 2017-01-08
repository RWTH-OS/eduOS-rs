;;; Based on http://blog.phil-opp.com/rust-os/entering-longmode.html
;;; and http://blog.phil-opp.com/rust-os/setup-rust.html
;;;
;;; Once we've run all our 32-bit setup code, we jump here and enter 64-bit
;;; mode.
;;;
;;; To generate yellow 4-letter debug text values, you can run:
;;;
;;; "INT!".chars.map {|c| sprintf("2f%02x", c.ord) }.reverse.join

%include 'common.inc'

global long_mode_start

extern rust_main

section .text
bits 64
long_mode_start:
        call setup_SSE
        call rust_main

        ;; Display "OKAY".
        mov rax, 0x2f592f412f4b2f4f
        mov qword [SCREEN_BASE], rax
        hlt

;;; Print "ERROR: " and an error code.
;;;
;;; a1: Error code.
error:
        mov rbx, 0x4f4f4f524f524f45
        mov [SCREEN_BASE], rbx
        mov rbx, 0x4f204f204f3a4f52
        mov [SCREEN_BASE + 8], rbx
        mov byte [SCREEN_BASE + 0xe], al
        hlt

;;; Check for SSE and enable it, or display an error.
;;;
;;; Copied from http://blog.phil-opp.com/rust-os/setup-rust.html, which got
;;; it from http://wiki.osdev.org/SSE#Checking_for_SSE.
setup_SSE:
        ;; Check for SSE.
        mov rax, 0x1
        cpuid
        test edx, 1<<25
        jz .no_SSE

        ;; Enable SSE.
        mov rax, cr0
        and ax, 0xFFFB      ; Clear coprocessor emulation CR0.EM.
        or ax, 0x2          ; Set coprocessor monitoring  CR0.MP.
        mov cr0, rax
        mov rax, cr4
        or ax, 3 << 9       ; Set CR4.OSFXSR and CR4.OSXMMEXCPT.
        mov cr4, rax

        ret
.no_SSE:
        mov al, "S"
        jmp error

