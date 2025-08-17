#![allow(dead_code)]
use core::arch::global_asm;

/* Save caller-saved regs + sepc/sstatus, call rust_trap(&mut TrapFrame), restore, sret.
   Frame layout (bytes, 8-byte slots):
   0: ra (x1)
   8: sp (x2)   <-- interrupted stack pointer
  16: t0 (x5)
  24: t1 (x6)
  32: t2 (x7)
  40: a0 (x10)
  48: a1 (x11)
  56: a2 (x12)
  64: a3 (x13)
  72: a4 (x14)
  80: a5 (x15)
  88: a6 (x16)
  96: a7 (x17)
 104: t3 (x28)
 112: t4 (x29)
 120: t5 (x30)
 128: t6 (x31)
 136: sepc
 144: sstatus
  Total size = 152 bytes
*/

global_asm!(
    r#"
    .section .text.trap
    .globl __trap_entry
__trap_entry:
    addi    sp, sp, -152
    sd      ra,   0(sp)
    sd      sp,   8(sp)     
    sd      t0,  16(sp)
    sd      t1,  24(sp)
    sd      t2,  32(sp)
    sd      a0,  40(sp)
    sd      a1,  48(sp)
    sd      a2,  56(sp)
    sd      a3,  64(sp)
    sd      a4,  72(sp)
    sd      a5,  80(sp)
    sd      a6,  88(sp)
    sd      a7,  96(sp)
    sd      t3, 104(sp)
    sd      t4, 112(sp)
    sd      t5, 120(sp)
    sd      t6, 128(sp)
    csrr    t0, sepc
    sd      t0, 136(sp)
    csrr    t0, sstatus
    sd      t0, 144(sp)

    mv      a0, sp           
    call    rust_trap

    ld      t0, 136(sp)
    csrw    sepc, t0
    ld      t0, 144(sp)
    csrw    sstatus, t0

    ld      ra,   0(sp)
    ld      t0,  16(sp)
    ld      t1,  24(sp)
    ld      t2,  32(sp)
    ld      a0,  40(sp)
    ld      a1,  48(sp)
    ld      a2,  56(sp)
    ld      a3,  64(sp)
    ld      a4,  72(sp)
    ld      a5,  80(sp)
    ld      a6,  88(sp)
    ld      a7,  96(sp)
    ld      t3, 104(sp)
    ld      t4, 112(sp)
    ld      t5, 120(sp)
    ld      t6, 128(sp)
    ld      sp,   8(sp)
    addi    sp, sp, 152
    sret
"#
);
