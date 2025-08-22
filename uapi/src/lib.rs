#![no_std]

pub mod nr {
    pub const WRITE: usize = 1; // write(ptr,len) -> usize
    pub const EXIT: usize = 2; // exit()
    pub const WRITE_CSTR: usize = 3; // write_cstr(ptr) -> usize
    pub const OPEN: usize = 4; // open_cstr(path) -> fd or usize::MAX
    pub const READ: usize = 5; // read(fd, buf, len) -> n or usize::MAX
    pub const WRITE_FD: usize = 6; // write(fd, buf, len) -> n or usize::MAX
    pub const CLOSE: usize = 7; // close(fd) -> 0 or usize::MAX
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum SysErr {
    Fail,
}

pub type SysResult<T> = core::result::Result<T, SysErr>;

#[inline(always)]
pub const fn is_err_sentinel(v: usize) -> bool {
    v == usize::MAX
}
