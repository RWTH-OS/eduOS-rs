OUTPUT_ARCH("riscv")
ENTRY(_start)

/* QEMU's `virt` machine starts execution of a kernel loaded with `-bios none`
 * at the beginning of its RAM, which is located at 0x80000000. */
phys = 0x80000000;

SECTIONS
{
  . = phys;
  kernel_start = .;
  .text : {
    *(.text._start)
    *(.text)
    *(.text.*)
  }
  .rodata : ALIGN(8) {
    *(.rodata)
    *(.rodata.*)
  }
  .data : ALIGN(8) {
    *(.data)
    *(.data.*)
    /* Place the global pointer so that the small data sections are reachable
     * with a signed 12-bit offset (+/- 2 KiB) relative to it. */
    . = ALIGN(8);
    PROVIDE(__global_pointer$ = . + 0x800);
    *(.sdata)
    *(.sdata.*)
  }
  .bss : ALIGN(8) {
    __bss_start = .;
    *(.sbss)
    *(.sbss.*)
    *(.bss)
    *(.bss.*)
    __bss_end = .;
  }
  . = ALIGN(16); /* RISC-V requires a 16-byte aligned stack pointer */
  /***********************************************************************************************
   * Boot Core Stack
   ***********************************************************************************************/
  __boot_core_stack_start = .;         /*   ^             */
                                       /*   | stack       */
  . += 32K;                            /*   | growth      */
                                       /*   | direction   */
  __boot_core_stack_end_exclusive = .; /*   |             */
}
