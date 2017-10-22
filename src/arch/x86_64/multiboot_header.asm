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

;;; Based on https://github.com/RWTH-OS/eduOS
;;;
;;; This is our Multiboot 1 header, which Grub uses to find our kernel code
;;; and load it into memory.

section .multiboot_header
bits 32
header_start:
        ; Multiboot macros to make a few lines more readable later
        MULTIBOOT_PAGE_ALIGN	equ (1 << 0)
        MULTIBOOT_MEMORY_INFO	equ (1 << 1)
        MULTIBOOT_HEADER_MAGIC	equ 0x1BADB002
        MULTIBOOT_HEADER_FLAGS	equ MULTIBOOT_PAGE_ALIGN | MULTIBOOT_MEMORY_INFO
        MULTIBOOT_CHECKSUM	equ -(MULTIBOOT_HEADER_MAGIC + MULTIBOOT_HEADER_FLAGS)

        ; This is the GRUB Multiboot header. A boot signature
        dd MULTIBOOT_HEADER_MAGIC
        dd MULTIBOOT_HEADER_FLAGS
        dd MULTIBOOT_CHECKSUM
        dd 0, 0, 0, 0, 0 ; address fields
