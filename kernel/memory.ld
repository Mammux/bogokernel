OUTPUT_ARCH(riscv)
ENTRY(_start)

MEMORY
{
  RAM (rwx) : ORIGIN = 0x80000000, LENGTH = 128M
}

KERNEL_LOAD_ADDR = 0x80200000;

SECTIONS
{
  .text KERNEL_LOAD_ADDR : ALIGN(4K) {
    KEEP(*(.text.entry))   /* our _start */
    *(.text .text.*)
  }

  .rodata : ALIGN(4K) {
    *(.rodata .rodata.*)
  }

  .data : ALIGN(4K) {
    *(.sdata .sdata.*)
    *(.data .data.*)
  }

  .bss : ALIGN(4K) {
    __bss_start = .;
    *(.sbss .sbss.*)
    *(.bss .bss.*)
    . = ALIGN(4K);
    __bss_end = .;
  }

  /* Simple, single big stack for hart0 for now */
  . = ORIGIN(RAM) + LENGTH(RAM);
  _stack_top = .;

  /* A simple contiguous heap region (unused yet) */
  __heap_start = __bss_end;
  __heap_end   = _stack_top - 0x10000; /* leave 64 KiB headroom for the stack */
}

PROVIDE(__global_pointer$ = 0); /* calm linker warnings for small data model */

