#![no_std]
#![no_main]

use core::ffi::CStr;
use usys::{creat, IoWrite};

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
    usys::println!("Creating test files...");
    
    // Create file1.txt
    let path1 = CStr::from_bytes_with_nul(b"file1.txt\0").unwrap();
    match creat(path1, 0o644) {
        Ok(fd) => {
            usys::println!("Created file1.txt");
            let _ = fd.write(b"This is file 1\n");
            let _ = fd.close();
        }
        Err(_) => {
            usys::println!("Failed to create file1.txt");
        }
    }
    
    // Create file2.txt
    let path2 = CStr::from_bytes_with_nul(b"file2.txt\0").unwrap();
    match creat(path2, 0o644) {
        Ok(fd) => {
            usys::println!("Created file2.txt");
            let _ = fd.write(b"This is file 2\n");
            let _ = fd.close();
        }
        Err(_) => {
            usys::println!("Failed to create file2.txt");
        }
    }
    
    // Create mydata.txt
    let path3 = CStr::from_bytes_with_nul(b"mydata.txt\0").unwrap();
    match creat(path3, 0o600) {
        Ok(fd) => {
            usys::println!("Created mydata.txt");
            let _ = fd.write(b"Some important data here\n");
            let _ = fd.close();
        }
        Err(_) => {
            usys::println!("Failed to create mydata.txt");
        }
    }
    
    usys::println!("Done creating files!");
    usys::exit();
}
