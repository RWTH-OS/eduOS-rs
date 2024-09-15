ENTRY(_start)
phys = 0x000000100000;

SECTIONS
{
  kernel_start = phys;
  .mboot phys : AT(ADDR(.mboot)) {
    KEEP(*(.mboot))
    KEEP(*(.mboot.*))
  }
  .text ALIGN(4096) : AT(ADDR(.text)) {
    *(.text)
    *(.text.*)
  }
  .rodata ALIGN(4096) : AT(ADDR(.rodata)) {
    *(.rodata)
    *(.rodata.*)
  }
  .data ALIGN(4096) : AT(ADDR(.data)) {
    *(.data)
    *(.data.*)
  }
  .bss ALIGN(4096) : AT(ADDR(.bss)) {
    *(.bss)
    *(.bss.*)
  }
}