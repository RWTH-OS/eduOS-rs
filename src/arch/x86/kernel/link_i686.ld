ENTRY(_start)
phys = 0x000000100000;

SECTIONS
{
  kernel_start = phys;
  .mboot phys : {
    KEEP(*(.mboot))
    KEEP(*(.mboot.*))
  }
  .text ALIGN(4) : {
    *(.text)
    *(.text.*)
  }
  .rodata ALIGN(4) : {
    *(.rodata)
    *(.rodata.*)
  }
  .data ALIGN(4) : {
    *(.data)
    *(.data.*)
  }
  .bss ALIGN(4) : {
    __bss_start = .;
    *(.bss)
    *(.bss.*)
    __bss_end = .;
  }
}