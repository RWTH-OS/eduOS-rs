/* Based on http://blog.phil-opp.com/rust-os/multiboot-kernel.html
 *
 * Used to specify a custom linking layout that puts our multiboot header
 * before everything else.
 */

OUTPUT_FORMAT("elf32-i386")
OUTPUT_ARCH("i386")
ENTRY(start)

SECTIONS {
	/* Load the kernel reasonably high in memory to avoid special addresses. */
	. = 1M;

	kernel_start = .;
	.boot :
	{
		/* This goes first. */
		KEEP(*(.multiboot_header))
	}

	.text ALIGN(4096) : AT(ADDR(.text))
	{
		*(.text)
		*(.text.*)
	}

	.rodata : AT(ADDR(.rodata))
	{
		*(.rodata)
		*(.rodata.*)
	}

	.data ALIGN(4096) : AT(ADDR(.data))
	{
		*(.data)
		*(.data.*)
	}

	.bss ALIGN(4096) : AT(ADDR(.bss))
	{
		__bss_start = .;
		*(.bss)
		*(.bss.*)
	}
	kernel_end = .;
}
