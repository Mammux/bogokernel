#![no_std]
#![no_main]

use core::ffi::CStr;
use usys::{cstr, exit, open, write, write_cstr};

#[no_mangle]
pub extern "C" fn _start(argc: usize, argv: *const *const u8, _envp: *const *const u8) -> ! {
    unsafe {
        let path: &CStr = if argc > 1 {
            let p = core::ptr::read(argv.add(1));
            // SAFETY: kernel provided a NUL-terminated argv; we trust our loader
            CStr::from_ptr(p)
        } else {
            cstr!("hello.txt")
        };

        match open(path) {
            Ok(fd) => {
                let mut buf = [0u8; 256];
                loop {
                    match fd.read(&mut buf) {
                        Ok(0) => break,
                        Ok(n) => {
                            let _ = write(&buf[..n]);
                        }
                        Err(_) => {
                            let _ = write_cstr(cstr!("cat: read error\n"));
                            break;
                        }
                    }
                }
                let _ = fd.close();
                let _ = write(b"\n");
            }
            Err(_) => {
                let _ = write_cstr(cstr!("cat: open failed\n"));
            }
        }

        exit();
    }
}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}
