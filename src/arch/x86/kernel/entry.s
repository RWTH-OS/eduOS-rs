# Muktiboot header, which is used by the loader to detect
# a multiboot kernel

.code32

# We use a special name to map this section at the begin of our kernel
# =>  Multiboot expects its magic number at the beginning of the kernel.
.section .mboot, "a"

# This part MUST be 4 byte aligned, so we solve that issue using '.align 4'.
.align 4
.global mboot
mboot:
    # Multiboot macros to make a few lines more readable later
    .set MULTIBOOT_PAGE_ALIGN,    (1 << 0)
    .set MULTIBOOT_MEMORY_INFO,   (1 << 1)
    .set MULTIBOOT_HEADER_MAGIC,  0x1BADB002
    .set MULTIBOOT_HEADER_FLAGS,  MULTIBOOT_PAGE_ALIGN | MULTIBOOT_MEMORY_INFO
    .set MULTIBOOT_CHECKSUM,      -(MULTIBOOT_HEADER_MAGIC + MULTIBOOT_HEADER_FLAGS)

    # This is the GRUB Multiboot header. A boot signature
    .4byte MULTIBOOT_HEADER_MAGIC
    .4byte MULTIBOOT_HEADER_FLAGS
    .4byte MULTIBOOT_CHECKSUM
    .4byte 0, 0, 0, 0, 0 # address fields