#![allow(dead_code)]
use core::arch::global_asm;

global_asm!(
    r#"
    .section .text.entry
    .globl _start
_start:
    /* Put SP a little below top-of-RAM */
    la   sp, _stack_top
    addi sp, sp, -16

    /* Early UART poke to prove we’re alive (16550 at 0x10000000) */
    li   t2, 0x10000000
1:  lb   t3, 5(t2)           /* LSR */
    andi t3, t3, 0x20        /* THR empty? bit 5 */
    beqz t3, 1b
    li   t3, '!'
    sb   t3, 0(t2)           /* write '!' */

    /* Zero .bss */
    la   t0, __bss_start
    la   t1, __bss_end
2:
    bgeu t0, t1, 3f
    sd   zero, 0(t0)
    addi t0, t0, 8
    j    2b
3:
    /* Jump to Rust */
    la   t0, rust_start
    jr   t0
"#
);
