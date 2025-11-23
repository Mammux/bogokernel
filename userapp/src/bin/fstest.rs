#![no_std]
#![no_main]

use core::ffi::CStr;
use usys::{creat, open, stat, unlink, chmod, IoWrite, IoRead, Fd};

#[panic_handler]
fn on_panic(_info: &core::panic::PanicInfo) -> ! {
    usys::println!("PANIC: {}", _info);
    usys::exit();
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    main();
}

fn main() -> ! {
    usys::println!("=== Filesystem Test ===");
    
    // Test 1: Create a new file
    usys::println!("\n[Test 1] Creating test.txt...");
    let path = CStr::from_bytes_with_nul(b"test.txt\0").unwrap();
    
    match creat(path, 0o644) {
        Ok(fd) => {
            usys::println!("✓ Created test.txt (fd={})", fd.0);
            
            // Test 2: Write to the file
            usys::println!("\n[Test 2] Writing to test.txt...");
            let data = b"Hello, writable filesystem!\n";
            match fd.write(data) {
                Ok(n) => {
                    usys::println!("✓ Wrote {} bytes", n);
                }
                Err(_) => {
                    usys::println!("✗ Failed to write");
                }
            }
            
            // Close the file
            let _ = fd.close();
        }
        Err(_) => {
            usys::println!("✗ Failed to create test.txt");
        }
    }
    
    // Test 3: Open and read the file
    usys::println!("\n[Test 3] Reading test.txt...");
    match open(path) {
        Ok(fd) => {
            usys::println!("✓ Opened test.txt (fd={})", fd.0);
            
            let mut buf = [0u8; 128];
            match fd.read(&mut buf) {
                Ok(n) => {
                    usys::println!("✓ Read {} bytes:", n);
                    if let Ok(s) = core::str::from_utf8(&buf[..n]) {
                        usys::println!("  Content: {}", s);
                    }
                }
                Err(_) => {
                    usys::println!("✗ Failed to read");
                }
            }
            
            let _ = fd.close();
        }
        Err(_) => {
            usys::println!("✗ Failed to open test.txt");
        }
    }
    
    // Test 4: stat() the file
    usys::println!("\n[Test 4] Checking file stats...");
    let mut stat_buf = [0u64; 2];
    match stat(path, &mut stat_buf) {
        Ok(_) => {
            usys::println!("✓ File exists");
            usys::println!("  Size: {} bytes", stat_buf[0]);
            usys::println!("  Mode: 0{:o}", stat_buf[1]);
        }
        Err(_) => {
            usys::println!("✗ stat() failed");
        }
    }
    
    // Test 5: chmod() the file
    usys::println!("\n[Test 5] Changing permissions...");
    match chmod(path, 0o400) {
        Ok(_) => {
            usys::println!("✓ chmod() succeeded");
            
            // Verify with stat
            match stat(path, &mut stat_buf) {
                Ok(_) => {
                    usys::println!("  New mode: 0{:o}", stat_buf[1]);
                }
                Err(_) => {}
            }
        }
        Err(_) => {
            usys::println!("✗ chmod() failed");
        }
    }
    
    // Test 6: unlink() the file
    usys::println!("\n[Test 6] Deleting test.txt...");
    match unlink(path) {
        Ok(_) => {
            usys::println!("✓ File deleted");
            
            // Verify it's gone
            match stat(path, &mut stat_buf) {
                Ok(_) => {
                    usys::println!("✗ File still exists!");
                }
                Err(_) => {
                    usys::println!("✓ File no longer exists");
                }
            }
        }
        Err(_) => {
            usys::println!("✗ unlink() failed");
        }
    }
    
    // Test 7: Create and write multiple times
    usys::println!("\n[Test 7] Multiple writes...");
    match creat(path, 0o644) {
        Ok(fd) => {
            let _ = fd.write(b"Line 1\n");
            let _ = fd.write(b"Line 2\n");
            let _ = fd.write(b"Line 3\n");
            let _ = fd.close();
            
            // Read it back
            match open(path) {
                Ok(fd) => {
                    let mut buf = [0u8; 128];
                    if let Ok(n) = fd.read(&mut buf) {
                        usys::println!("✓ Multiple writes successful:");
                        if let Ok(s) = core::str::from_utf8(&buf[..n]) {
                            usys::println!("{}", s);
                        }
                    }
                    let _ = fd.close();
                }
                Err(_) => {}
            }
            
            let _ = unlink(path);
        }
        Err(_) => {
            usys::println!("✗ Failed to create file");
        }
    }
    
    usys::println!("\n=== All tests complete ===\n");
    usys::exit();
}
