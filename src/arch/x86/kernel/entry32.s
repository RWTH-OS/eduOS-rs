# This is the kernel's entry point. We could either call main here,
# or we can use this to setup the stack or other nice stuff, like
# perhaps setting up the GDT and segments. Please note that interrupts
# are disabled at this point: More on interrupts later!

.code32

# We use a special name to map this section at the begin of our kernel
# =>  Multiboot expects its magic number at the beginning of the kernel.
.section .mboot, "a"

# This part MUST be 4 byte aligned, so we solve that issue using '.align 4'.
.align 4
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