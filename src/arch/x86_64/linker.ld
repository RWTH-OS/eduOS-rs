/* Based on http://blog.phil-opp.com/rust-os/multiboot-kernel.html
 *
 * Used to specify a custom linking layout that puts our multiboot header
 * before everything else.
 */

OUTPUT_FORMAT("elf64-x86-64")
OUTPUT_ARCH("i386:x86-64")
ENTRY(start)

SECTIONS {
    /* Load the kernel reasonably high in memory to avoid special addresses. */
    . = 1M;

    .boot :
    {
        /* This goes first. */
        KEEP(*(.multiboot_header))
    }

    .text ALIGN(4096) : AT(ADDR(.text))
    {
        *(.text)
    }
}
