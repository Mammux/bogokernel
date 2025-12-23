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
        name: "mkfiles.elf",
        data: include_bytes!("../mkfiles.elf"),
    },
    File {
        name: "gputest.elf",
        data: include_bytes!("../gputest.elf"),
    },
    File {
        name: "forth.elf",
        data: include_bytes!("../forth.elf"),
    },
    File {
        name: "lisp.elf",
        data: include_bytes!("../lisp.elf"),
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

/// List writable files - returns number of files and writes names to buffer
/// Each filename is null-terminated in the buffer
pub fn list_writable_files(buf: &mut [u8]) -> usize {
    let files = WRITABLE_FILES.lock();
    let mut offset = 0usize;
    let mut count = 0usize;
    
    for file in files.iter() {
        let name_bytes = file.name.as_bytes();
        // +1 for null terminator
        if offset + name_bytes.len() + 1 > buf.len() {
            break; // Buffer full
        }
        
        // Copy filename
        buf[offset..offset + name_bytes.len()].copy_from_slice(name_bytes);
        offset += name_bytes.len();
        
        // Add null terminator
        buf[offset] = 0;
        offset += 1;
        
        count += 1;
    }
    
    count
}

/// Initialize writable filesystem with embedded files
/// This moves all files from the read-only RAMFS to the writable filesystem
pub fn init_writable_fs() {
    let mut files = WRITABLE_FILES.lock();
    
    // Copy all embedded files to writable filesystem
    for file in FILES {
        files.push(WritableFile {
            name: String::from(file.name),
            data: Vec::from(file.data),
            mode: if file.name.ends_with(".elf") { 0o755 } else { 0o644 },
        });
    }
}

/// Lookup a file by name in the writable filesystem
/// Returns a copy of the file data if found
/// 
/// Note: This function clones the file data to avoid holding the filesystem
/// lock during ELF loading. While this involves copying, it's necessary because:
/// 1. ELF loading is a long operation that cannot hold the lock
/// 2. File data must remain stable during the entire loading process
/// 3. Program execution is not a hot path, so the overhead is acceptable
pub fn get_file_data(name: &str) -> Option<Vec<u8>> {
    let files = WRITABLE_FILES.lock();
    files.iter().find(|f| f.name == name).map(|f| f.data.clone())
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;

    // Helper to reset filesystem state between tests
    fn reset_fs() {
        let mut files = WRITABLE_FILES.lock();
        files.clear();
    }

    #[test]
    fn test_create_file() {
        reset_fs();
        
        let result = create_file("test.txt");
        assert!(result.is_ok());
        
        let idx = result.unwrap();
        assert_eq!(idx, 0);
        
        // Verify file was created
        assert!(lookup_writable("test.txt").is_some());
    }

    #[test]
    fn test_create_file_truncates_existing() {
        reset_fs();
        
        // Create file with some data
        let idx1 = create_file("test.txt").unwrap();
        write_file(idx1, 0, b"Hello").unwrap();
        
        // Create again (should truncate)
        let idx2 = create_file("test.txt").unwrap();
        assert_eq!(idx1, idx2);
        
        // Verify file is empty
        let size = file_size(idx2);
        assert_eq!(size, Some(0));
    }

    #[test]
    fn test_write_and_read_file() {
        reset_fs();
        
        let idx = create_file("test.txt").unwrap();
        let data = b"Hello, World!";
        
        // Write data
        let written = write_file(idx, 0, data).unwrap();
        assert_eq!(written, data.len());
        
        // Read data back
        let mut buf = vec![0u8; data.len()];
        let read = read_file(idx, 0, &mut buf).unwrap();
        assert_eq!(read, data.len());
        assert_eq!(&buf[..], data);
    }

    #[test]
    fn test_write_at_offset() {
        reset_fs();
        
        let idx = create_file("test.txt").unwrap();
        
        // Write at offset 0
        write_file(idx, 0, b"Hello").unwrap();
        
        // Write at offset 5 (should extend file with zeros)
        write_file(idx, 5, b" World").unwrap();
        
        // Read entire file
        let mut buf = vec![0u8; 11];
        let read = read_file(idx, 0, &mut buf).unwrap();
        assert_eq!(read, 11);
        assert_eq!(&buf[..], b"Hello World");
    }

    #[test]
    fn test_read_at_offset() {
        reset_fs();
        
        let idx = create_file("test.txt").unwrap();
        write_file(idx, 0, b"Hello, World!").unwrap();
        
        // Read from middle of file
        let mut buf = vec![0u8; 5];
        let read = read_file(idx, 7, &mut buf).unwrap();
        assert_eq!(read, 5);
        assert_eq!(&buf[..], b"World");
    }

    #[test]
    fn test_read_beyond_end() {
        reset_fs();
        
        let idx = create_file("test.txt").unwrap();
        write_file(idx, 0, b"Hello").unwrap();
        
        // Try to read beyond file end
        let mut buf = vec![0u8; 10];
        let read = read_file(idx, 5, &mut buf).unwrap();
        assert_eq!(read, 0);
    }

    #[test]
    fn test_read_partial() {
        reset_fs();
        
        let idx = create_file("test.txt").unwrap();
        write_file(idx, 0, b"Hello").unwrap();
        
        // Read with buffer smaller than file
        let mut buf = vec![0u8; 3];
        let read = read_file(idx, 0, &mut buf).unwrap();
        assert_eq!(read, 3);
        assert_eq!(&buf[..], b"Hel");
    }

    #[test]
    fn test_file_size() {
        reset_fs();
        
        let idx = create_file("test.txt").unwrap();
        
        // Empty file
        assert_eq!(file_size(idx), Some(0));
        
        // After writing
        write_file(idx, 0, b"Hello").unwrap();
        assert_eq!(file_size(idx), Some(5));
        
        // After writing at offset (extends file)
        write_file(idx, 10, b"World").unwrap();
        assert_eq!(file_size(idx), Some(15));
    }

    #[test]
    fn test_unlink_file() {
        reset_fs();
        
        create_file("test.txt").unwrap();
        assert!(lookup_writable("test.txt").is_some());
        
        // Delete file
        let result = unlink_file("test.txt");
        assert!(result.is_ok());
        
        // Verify file is gone
        assert!(lookup_writable("test.txt").is_none());
    }

    #[test]
    fn test_unlink_nonexistent() {
        reset_fs();
        
        let result = unlink_file("nonexistent.txt");
        assert!(result.is_err());
    }

    #[test]
    fn test_chmod_file() {
        reset_fs();
        
        create_file("test.txt").unwrap();
        
        // Change mode
        let result = chmod_file("test.txt", 0o755);
        assert!(result.is_ok());
        
        // Verify mode changed
        let stat = stat_file("test.txt").unwrap();
        assert_eq!(stat.mode, 0o755);
    }

    #[test]
    fn test_chmod_nonexistent() {
        reset_fs();
        
        let result = chmod_file("nonexistent.txt", 0o755);
        assert!(result.is_err());
    }

    #[test]
    fn test_file_exists() {
        reset_fs();
        
        assert!(!file_exists("test.txt"));
        
        create_file("test.txt").unwrap();
        assert!(file_exists("test.txt"));
    }

    #[test]
    fn test_stat_file() {
        reset_fs();
        
        let idx = create_file("test.txt").unwrap();
        write_file(idx, 0, b"Hello").unwrap();
        
        let stat = stat_file("test.txt").unwrap();
        assert_eq!(stat.size, 5);
        assert_eq!(stat.mode, 0o600);
        assert!(stat.is_writable);
    }

    #[test]
    fn test_stat_nonexistent() {
        reset_fs();
        
        let stat = stat_file("nonexistent.txt");
        assert!(stat.is_none());
    }

    #[test]
    fn test_multiple_files() {
        reset_fs();
        
        // Create multiple files
        let idx1 = create_file("file1.txt").unwrap();
        let idx2 = create_file("file2.txt").unwrap();
        let idx3 = create_file("file3.txt").unwrap();
        
        // Write different data to each
        write_file(idx1, 0, b"File 1").unwrap();
        write_file(idx2, 0, b"File 2").unwrap();
        write_file(idx3, 0, b"File 3").unwrap();
        
        // Read and verify each file
        let mut buf = vec![0u8; 10];
        
        read_file(idx1, 0, &mut buf).unwrap();
        assert_eq!(&buf[..6], b"File 1");
        
        read_file(idx2, 0, &mut buf).unwrap();
        assert_eq!(&buf[..6], b"File 2");
        
        read_file(idx3, 0, &mut buf).unwrap();
        assert_eq!(&buf[..6], b"File 3");
    }

    #[test]
    fn test_lookup_writable() {
        reset_fs();
        
        assert!(lookup_writable("test.txt").is_none());
        
        create_file("test.txt").unwrap();
        let idx = lookup_writable("test.txt");
        assert!(idx.is_some());
        assert_eq!(idx.unwrap(), 0);
    }

    #[test]
    fn test_write_file_invalid_index() {
        reset_fs();
        
        let result = write_file(999, 0, b"data");
        assert!(result.is_err());
    }

    #[test]
    fn test_read_file_invalid_index() {
        reset_fs();
        
        let mut buf = vec![0u8; 10];
        let result = read_file(999, 0, &mut buf);
        assert!(result.is_err());
    }

    #[test]
    fn test_file_size_invalid_index() {
        reset_fs();
        
        let size = file_size(999);
        assert!(size.is_none());
    }
}
