// kernel/src/fs.rs
#![allow(dead_code)]

pub struct File {
    pub name: &'static str,
    pub data: &'static [u8],
}

// Embed a couple of files; add more as you like.
static HELLO_TXT: &[u8] = b"Hello from RAMFS!\n";
static ETC_MOTD:  &[u8] = b"Welcome to BogoKernel.\n";

pub static FILES: &[File] = &[
    File { name: "hello.txt", data: HELLO_TXT }, 
    File { name: "etc/motd",  data: ETC_MOTD  },
];

pub fn lookup(name: &str) -> Option<&'static File> {
    for f in FILES {
        if f.name == name { return Some(f); }
    }
    None
}
