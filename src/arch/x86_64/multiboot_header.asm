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
