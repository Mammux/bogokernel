// kernel/src/fs.rs
#![allow(dead_code)]

use alloc::vec::Vec;
use alloc::string::String;
use spin::Mutex;

pub struct File {
    pub name: &'static str,
    pub data: &'static [u8],
}

// Embed a couple of files; add more as you like.
static HELLO_TXT: &[u8] = b"Hello from RAMFS!\n";
static ETC_MOTD: &[u8] = b"Welcome to BogoKernel.\n";

pub static FILES: &[File] = &[
    File {
        name: "dungeon.map",
        data: include_bytes!("dungeon.map"),
    },
    File {
        name: "shell.elf",
        data: include_bytes!("../shell.elf"),
    },
    File {
        name: "rogue.elf",
        data: include_bytes!("../rogue.elf"),
    },
    File {
        name: "hello.elf",
        data: include_bytes!("../hello.elf"),
    },
    File {
        name: "crogue.elf",
        data: include_bytes!("../crogue.elf"),
    },
    File {
        name: "curses_test.elf",
        data: include_bytes!("../curses_test.elf"),
    },
    File {
        name: "bigrogue.elf",
        data: include_bytes!("../bigrogue.elf"),
    },
    File {
        name: "fstest.elf",
        data: include_bytes!("../fstest.elf"),
    },
    File {
        name: "etc/motd",
        data: ETC_MOTD,
    },
];


pub fn lookup(name: &str) -> Option<&'static File> {
    for f in FILES {
        if f.name == name {
            return Some(f);
        }
    }
    None
}

// --- Writable filesystem layer ---

/// A writable file stored in kernel memory
pub struct WritableFile {
    pub name: String,
    pub data: Vec<u8>,
    pub mode: u32,
}

static WRITABLE_FILES: Mutex<Vec<WritableFile>> = Mutex::new(Vec::new());

/// Create or truncate a writable file
pub fn create_file(name: &str) -> Result<usize, ()> {
    let mut files = WRITABLE_FILES.lock();
    
    // Check if file already exists
    for (idx, f) in files.iter().enumerate() {
        if f.name == name {
            // Truncate existing file
            files[idx].data.clear();
            files[idx].mode = 0o600;
            return Ok(idx);
        }
    }
    
    // Create new file
    files.push(WritableFile {
        name: String::from(name),
        data: Vec::new(),
        mode: 0o600,
    });
    Ok(files.len() - 1)
}

/// Lookup a writable file by name, returns index
pub fn lookup_writable(name: &str) -> Option<usize> {
    let files = WRITABLE_FILES.lock();
    files.iter().position(|f| f.name == name)
}

/// Write data to a writable file at the given offset
pub fn write_file(idx: usize, offset: usize, data: &[u8]) -> Result<usize, ()> {
    let mut files = WRITABLE_FILES.lock();
    if idx >= files.len() {
        return Err(());
    }
    
    let file = &mut files[idx];
    
    // Extend file if needed
    let end_pos = offset + data.len();
    if end_pos > file.data.len() {
        file.data.resize(end_pos, 0);
    }
    
    // Write data
    file.data[offset..end_pos].copy_from_slice(data);
    Ok(data.len())
}

/// Read data from a writable file
pub fn read_file(idx: usize, offset: usize, buf: &mut [u8]) -> Result<usize, ()> {
    let files = WRITABLE_FILES.lock();
    if idx >= files.len() {
        return Err(());
    }
    
    let file = &files[idx];
    if offset >= file.data.len() {
        return Ok(0);
    }
    
    let available = &file.data[offset..];
    let to_read = core::cmp::min(buf.len(), available.len());
    buf[..to_read].copy_from_slice(&available[..to_read]);
    Ok(to_read)
}

/// Get the size of a writable file
pub fn file_size(idx: usize) -> Option<usize> {
    let files = WRITABLE_FILES.lock();
    files.get(idx).map(|f| f.data.len())
}

/// Delete a writable file
pub fn unlink_file(name: &str) -> Result<(), ()> {
    let mut files = WRITABLE_FILES.lock();
    if let Some(idx) = files.iter().position(|f| f.name == name) {
        files.remove(idx);
        Ok(())
    } else {
        Err(())
    }
}

/// Change file mode/permissions
pub fn chmod_file(name: &str, mode: u32) -> Result<(), ()> {
    let mut files = WRITABLE_FILES.lock();
    if let Some(f) = files.iter_mut().find(|f| f.name == name) {
        f.mode = mode;
        Ok(())
    } else {
        Err(())
    }
}

/// Check if a file exists (writable or read-only)
pub fn file_exists(name: &str) -> bool {
    // Check writable files first
    if lookup_writable(name).is_some() {
        return true;
    }
    // Check read-only files
    lookup(name).is_some()
}

/// Get file metadata
pub struct FileStat {
    pub size: usize,
    pub mode: u32,
    pub is_writable: bool,
}

pub fn stat_file(name: &str) -> Option<FileStat> {
    // Check writable files first
    let files = WRITABLE_FILES.lock();
    if let Some(f) = files.iter().find(|f| f.name == name) {
        return Some(FileStat {
            size: f.data.len(),
            mode: f.mode,
            is_writable: true,
        });
    }
    drop(files);
    
    // Check read-only files
    if let Some(f) = lookup(name) {
        return Some(FileStat {
            size: f.data.len(),
            mode: 0o444, // read-only
            is_writable: false,
        });
    }
    
    None
}
