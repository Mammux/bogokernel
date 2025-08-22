#![allow(dead_code)]
use core::arch::global_asm;

// A tiny U-mode program: print 'A' then exit.
// No data/PC-relative refs, so we can copy bytes anywhere safely.
global_asm!(r#"
    .section .userblob,"ax",@progbits
    .globl __user_blob_start
__user_blob_start:
    li  a0, 'A'        // syscall write(byte='A')
    li  a7, 1
    ecall

    li  a7, 2          // syscall exit()
    ecall

1:  wfi                // shouldn't return; if it does, idle
    j   1b

    .globl __user_blob_end
__user_blob_end:
"#);
