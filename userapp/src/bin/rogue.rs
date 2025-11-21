#![no_std]
#![no_main]

use usys::{cstr, println, print, IoRead};

#[no_mangle]
pub extern "C" fn _start(_argc: usize, _argv: *const *const u8, _envp: *const *const u8) -> ! {
    main();
    usys::exit();
}

fn main() {
    println!("Welcome to BogoHack!");
    println!("Loading dungeon...");

    let fd = match usys::open(cstr!("dungeon.map")) {
        Ok(fd) => fd,
        Err(_) => {
            println!("Error: Could not open dungeon.map");
            return;
        }
    };

    let mut map_data = [0u8; 4096];
    let mut offset = 0;
    loop {
        if offset >= map_data.len() {
            println!("Warning: Map too large, truncated");
            break;
        }
        match fd.read(&mut map_data[offset..]) {
            Ok(0) => break, // EOF
            Ok(n) => {
                offset += n;
            }
            Err(_) => {
                println!("Error: Could not read dungeon.map");
                return;
            }
        }
    }
    let n = offset;
    let _ = fd.close();

    if n == 0 {
        println!("Error: Dungeon map is empty");
        return;
    }

    let map_str = core::str::from_utf8(&map_data[..n]).unwrap_or("");
    
    // Find dimensions and player start
    let mut player_x = 0;
    let mut player_y = 0;
    let mut rows = 0;
    let mut cols = 0;
    
    for (y, line) in map_str.lines().enumerate() {
        rows = y + 1;
        if line.len() > cols { cols = line.len(); }
        if let Some(x) = line.find('<') {
            player_x = x;
            player_y = y;
        }
    }
    let width = cols;
    let height = rows;

    // Game Loop
    loop {
        // ANSI Clear Screen and Home
        print!("\x1b[2J\x1b[H");
        
        // Draw Map
        for (y, line) in map_str.lines().enumerate() {
            if y == player_y {
                // Draw line with player
                for (x, ch) in line.chars().enumerate() {
                    if x == player_x {
                        print!("@");
                    } else {
                        print!("{}", ch);
                    }
                }
                println!();
            } else {
                println!("{}", line);
            }
        }
        
        println!("Pos: ({}, {}) - Use WASD to move, Q to quit.", player_x, player_y);

        // Read Input
        let mut input = [0u8; 1];
        if let Ok(1) = usys::STDIN.read(&mut input) {
            let ch = input[0];
            let mut next_x = player_x;
            let mut next_y = player_y;

            match ch {
                b'w' => if next_y > 0 { next_y -= 1; },
                b's' => if next_y < height - 1 { next_y += 1; },
                b'a' => if next_x > 0 { next_x -= 1; },
                b'd' => if next_x < width - 1 { next_x += 1; },
                b'q' => break,
                _ => {},
            }

            // Collision Check
            // Re-parse map to check collision (inefficient but simple)
            let mut valid = false;
            if let Some(line) = map_str.lines().nth(next_y) {
                if let Some(c) = line.chars().nth(next_x) {
                    if c == '.' || c == '<' || c == '>' || c == ' ' {
                        valid = true;
                    }
                }
            }

            if valid {
                player_x = next_x;
                player_y = next_y;
            }
        }
    }
    
    println!("Game Over. Thanks for playing!");
}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

