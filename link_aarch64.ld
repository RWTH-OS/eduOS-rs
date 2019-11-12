ENTRY(_start)

SECTIONS {
	/* Load the kernel into ram of qemu device. */
	. = 0x40000000;

	kernel_start = .;

	.boot :
	{
		/* This goes first. */
		KEEP(*(.kheader))
	}

	.text : AT(ADDR(.text))
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