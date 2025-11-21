// kernel/src/fs.rs
#![allow(dead_code)]

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
        name: "simple_test.elf",
        data: include_bytes!("../simple_test.elf"),
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
