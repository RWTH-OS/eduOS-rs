%include 'common.inc'

global report_interrupt
global interrupt_handlers

extern rust_interrupt_handler

section .text
bits 64

;;; Call as `TRACE 'x'` to trace our progress through an interrupt handler.
%macro TRACE 1
        mov eax, %1
        jmp trace
%endmacro

;;; Prints "INTx" in green, where "x" is a character literal passed at the
;;; bottom of eax.
trace:
        shl eax, 16
        or eax, 0x2f202f54
        mov dword [SCREEN_BASE], 0x2f4e2f49
        mov dword [SCREEN_BASE + 0x4], eax
        hlt

;;; Registers to save pieced together from:
;;;
;;; http://stackoverflow.com/questions/6837392/how-to-save-the-registers-on-x86-64-for-an-interrupt-service-routine
;;; https://github.com/torvalds/linux/blob/master/arch/x86/entry/entry_64.S
;;; http://x86-64.org/documentation/abi.pdf
;;; http://developer.amd.com/wordpress/media/2012/10/24593_APM_v21.pdf
;;; https://github.com/redox-os/redox/blob/master/kernel/asm/interrupts-x86_64.asm
;;;
;;; We skip any "callee saved" registers, on the theory that the Rust
;;; compiler will save them if it actually uses them.
;;;
;;; We don't save any floating point, MMX, SSE, etc. registers, because
;;; they're large, complicated, and slow to save, and we want our interrupt
;;; handlers to be fast.  So we just don't use any of those processor
;;; features in kernel mode.
;;;
;;; This needs to be kept in sync with InterruptContext.
%macro push_caller_saved 0
        ;; For later: "<Tobba> ekidd: one thing you can also do to preserve
        ;; FPU registers in interrupt handlers is to simply set the EM flag in
        ;; CR0".  Seen on #rust-osdev.

        ;; Save ordinary registers.
        push rax
        push rcx
        push rdx
        push r8
        push r9
        push r10
        push r11
        ;; These two are caller-saved on x86_64!
        push rdi
        push rsi
%endmacro

%macro pop_caller_saved 0
        ;; Restore ordinary registers.
        pop rsi
        pop rdi
        pop r11
        pop r10
        pop r9
        pop r8
        pop rdx
        pop rcx
        pop rax
%endmacro

;;; "Error" interrupts have two extra dwords pushed onto the stack: a 0 pad to
;;; keep things aligned, and the the error code. See
;;; http://developer.amd.com/wordpress/media/2012/10/24593_APM_v21.pdf
;;; p. 247 "8.9 Long-Mode Interrupt Control Transfers".
;;;
;;; This needs to be kept in sync with InterruptContext.
%macro int_entry_error 1
int_entry_%1:
        ;; There's already qword error code here, which we're reponsible
        ;; for popping before IRET.
        push qword %1           ; Record interrupt ID.
        jmp int_shared          ; Now do the hard work for this interrupt.
%endmacro

;;; For non-error interrupts, we push an error code of zero for consistency.
;;;
;;; This needs to be kept in sync with InterruptContext.
%macro int_entry_dummy_error 1
int_entry_%1:
        push qword 0            ; Push error code of 0.
        push qword %1           ; Record interrupt ID.
        jmp int_shared          ; Now do the hard work for this interrupt.
%endmacro

;;; Interrupt names from
;;; http://developer.amd.com/wordpress/media/2012/10/24593_APM_v21.pdf
;;;
;;; You can double-check which ones have errors, and which ones don't, here:
;;; http://cgit.haiku-os.org/haiku/plain/src/system/kernel/arch/x86/64/interrupts.S
;;;
;;; 0 Integer Divide-by-Zero Exception
int_entry_dummy_error 0
;;; 1 Debug Exception
int_entry_dummy_error 1
;;; 2 Non-Maskable-Interrupt
int_entry_dummy_error 2
;;; 3 Breakpoint Exception (INT 3)
int_entry_dummy_error 3
;;; 4 Overflow Exception (INTO instruction)
int_entry_dummy_error 4
;;; 5 Bound-Range Exception (BOUND instruction)
int_entry_dummy_error 5
;;; 6 Invalid-Opcode Exception
int_entry_dummy_error 6
;;; 7 Device-Not-Available Exception
int_entry_dummy_error 7
;;; 8 Double-Fault Exception
int_entry_error 8                ; Error code is always 0?
;;; 9 Coprocessor-Segment-Overrun Exception (reserved in AMD64)
;;; 10 Invalid-TSS Exception
int_entry_error 10
;;; 11 Segment-Not-Present Exception
int_entry_error 11
;;; 12 Stack Exception
int_entry_error 12
;;; 13 General-Protection Exception
int_entry_error 13
;;; 14 Page-Fault Exception
int_entry_error 14               ; Maybe?
;;; 15 (Reserved)
;;; 16 x87 Floating-Point Exception
int_entry_dummy_error 16
;;; 17 Alignment-Check Exception
int_entry_error 17               ; Error code is always 0?
;;; 18 Machine-Check Exception
int_entry_dummy_error 18
;;; 19 SIMD Floating-Point Exception
int_entry_dummy_error 19
;;; 30 Security Exception

;;; Fill in cutom handlers 32 through 255.
int_entry_dummy_error 32
%assign i 33
%rep    224
int_entry_dummy_error i
%assign i i+1
%endrep

;;; All of the interrupt table entries wind up here, and we call into Rust.
int_shared:
        push_caller_saved

        mov rdi, rsp            ; Pass pointer to interrupt data.
        call rust_interrupt_handler

        pop_caller_saved
        add rsp, 16             ; Remove err code & interrupt ID.
        iretq

;;; A dummy interrupt handler.
report_interrupt:
        push_caller_saved

        ;; Print "INT!"
        mov rax, 0x2f212f542f4e2f49
        mov qword [SCREEN_BASE], rax

        pop_caller_saved
        iretq

section .rodata
interrupt_handlers:
        dq int_entry_0
        dq int_entry_1
        dq int_entry_2
        dq int_entry_3
        dq int_entry_4
        dq int_entry_5
        dq int_entry_6
        dq int_entry_7
        dq int_entry_8
        dq 0
        dq int_entry_10
        dq int_entry_11
        dq int_entry_12
        dq int_entry_13
        dq int_entry_14
        dq 0
        dq int_entry_16
        dq int_entry_17
        dq int_entry_18
        dq int_entry_19
        dq 0
        dq 0
        dq 0
        dq 0
        dq 0
        dq 0
        dq 0
        dq 0
        dq 0
        dq 0
        dq 0                    ; int_entry_30
        dq 0
%assign i 32
%rep    224
        dq int_entry_%+i
%assign i i+1
%endrep
