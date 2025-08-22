#![no_std]
#![no_main]

#[inline(always)]
unsafe fn sys_write(buf: *const u8, len: usize) -> usize {
    let mut ret: usize;
    core::arch::asm!(
        "ecall",
        in("a7") 1usize,       // write(ptr,len)
        in("a0") buf,
        in("a1") len,
        lateout("a0") ret,
        options(nostack),
    );
    ret
}
#[inline(always)]
unsafe fn sys_write_cstr(s: *const u8) -> usize {
    let mut ret: usize;
    core::arch::asm!(
        "ecall",
        in("a7") 3usize,       // write_cstr(ptr) — you added this
        in("a0") s,
        lateout("a0") ret,
        options(nostack),
    );
    ret
}
#[inline(always)]
unsafe fn sys_exit() -> ! {
    core::arch::asm!("ecall", in("a7") 2usize, options(noreturn, nostack));
}

/* ---- tiny formatting helpers (no heap) ---- */
unsafe fn put(s: &str) { let _ = sys_write(s.as_ptr(), s.len()); }
unsafe fn _put_hex_u64(x: u64) {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut buf = [0u8; 18]; // "0x" + 16 nybbles
    buf[0] = b'0'; buf[1] = b'x';
    for i in 0..16 {
        let nyb = (x >> ((15 - i) * 4)) & 0xF;
        buf[2 + i] = HEX[nyb as usize];
    }
    let _ = sys_write(buf.as_ptr(), buf.len());
}
unsafe fn _put_usize(x: usize) { _put_hex_u64(x as u64); } // hex is simplest & unambiguous
unsafe fn put_ln() { let _ = sys_write(b"\n".as_ptr(), 1); }

/* ---- the entry point ---- */
#[no_mangle]
pub extern "C" fn _start(argc: usize, argv: *const *const u8, _envp: *const *const u8) -> ! {
    unsafe {
        // put("DEBUG argc="); put_usize(argc); put_ln();

        // put("argv ptr="); put_usize(argv as usize); put("  envp ptr="); put_usize(envp as usize); put_ln();

        // Dump argv pointers + strings
        /*
        for i in 0..argc {
            let p = core::ptr::read(argv.add(i)); // p: *const u8 (cstr)
            put("argv["); put_usize(i); put("]="); put_usize(p as usize); put("  ");
            if !p.is_null() { let _ = sys_write_cstr(p); } else { put("(null)"); }
            put_ln();
        }
        */

        // Dump first few envp entries until NULL (cap to avoid infinite loops)
        /* 
        let mut j = 0usize;
        put("envp:"); put_ln();
        while j < 16 { // cap at 16 entries for safety
            let p = core::ptr::read(envp.add(j)); // p: *const u8 (cstr)
            if p.is_null() { break; }
            put("  envp["); put_usize(j); put("]="); put_usize(p as usize); put("  ");
            let _ = sys_write_cstr(p);
            put_ln();
            j += 1;
        }
        */

        // Minimal “normal” output (so you still see old behavior)
        put("ARGS: ");
        for i in 0..argc {
            let p = core::ptr::read(argv.add(i));
            if !p.is_null() {
                let _ = sys_write_cstr(p);
                let _ = sys_write(b" ".as_ptr(), 1);
            }
        }
        put_ln();

        sys_exit();
    }
}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! { loop {} }
