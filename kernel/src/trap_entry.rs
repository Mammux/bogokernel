use core::arch::global_asm;

global_asm!(
    r#"
    .section .text.trap
    .globl __trap_entry
__trap_entry:
    // Swap to kernel trap stack: sp <-> sscratch
    csrrw   sp, sscratch, sp

    // Make space for TrapFrame on the *kernel* stack
    addi    sp, sp, -152

    // Save registers (callee/caller selection kept minimal)
    sd      ra,   0(sp)

    // Save *user* SP that we just moved into sscratch
    csrr    t0, sscratch
    sd      t0,   8(sp)

    sd      t0,  16(sp)    // t0 (x5) — we’ll overwrite t0 next, so save a copy
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

    // Call Rust handler: rust_trap(&mut TrapFrame)
    mv      a0, sp
    call    rust_trap

    // Restore CSRs from TrapFrame (may be modified by handler)
    ld      t0, 136(sp)
    csrw    sepc, t0
    ld      t0, 144(sp)
    csrw    sstatus, t0

    // Restore GPRs
    ld      ra,   0(sp)
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

    // Recover the *user* SP from TrapFrame slot
    ld      t0,   8(sp)

    // Pop TrapFrame from kernel stack
    addi    sp, sp, 152

    // Put user SP back into sscratch, then swap stacks back (kernel->user)
    csrw    sscratch, t0
    csrrw   sp, sscratch, sp

    // Return to the privilege level indicated by saved sstatus.SPP (usually U)
    sret
"#
);
