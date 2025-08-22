#![no_std]
#![no_main]

use usys::{println, eprintln, cstr, CStrBuf, read_line_stdin, open, IoRead, STDOUT, exit};

#[no_mangle]
pub extern "C" fn _start(_argc: usize, _argv: *const *const u8, _envp: *const *const u8) -> ! {
        println!("enter a path (default: hello.txt): ");
        let mut line = [0u8; 256];
        let n = read_line_stdin(&mut line).unwrap_or(0);

        // choose default if empty
        let path_cstr = if n == 0 {
            cstr!("hello.txt")
        } else {
            // trim trailing spaces (optional)
            let mut end = n;
            while end > 0 && (line[end-1] == b' ' || line[end-1] == b'\t') { end -= 1; }
            let s = core::str::from_utf8(&line[..end]).unwrap_or("");
            // Build an owned C string
            let owned = CStrBuf::<256>::from_str(s).unwrap();
            // Keep it in scope while using it:
            let c = owned.as_cstr();
            // Use it right away (or store `owned`)
            match open(c) {
                Ok(fd) => {
                    let mut buf = [0u8; 256];
                    loop {
                        match fd.read(&mut buf) {
                            Ok(0) => break,
                            Ok(m) => { let _ = STDOUT.write(&buf[..m]); }
                            Err(_) => { eprintln!("read error"); break; }
                        }
                    }
                }
                Err(_) => eprintln!("open failed"),
            }
            exit(); // early exit to keep example short
        };

        // If empty line, default path:
        match open(path_cstr) {
            Ok(fd) => {
                let mut buf = [0u8; 256];
                loop {
                    match fd.read(&mut buf) {
                        Ok(0) => break,
                        Ok(m) => { let _ = STDOUT.write(&buf[..m]); }
                        Err(_) => { eprintln!("read error"); break; }
                    }
                }
            }
            Err(_) => eprintln!("open failed"),
        }

        exit();
    }

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}