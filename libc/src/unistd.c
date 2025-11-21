/* libc/src/unistd.c - POSIX syscall wrappers */
#include <unistd.h>

/* Syscall numbers */
#define SYS_WRITE      1
#define SYS_EXIT       2
#define SYS_OPEN       4
#define SYS_READ       5
#define SYS_WRITE_FD   6
#define SYS_CLOSE      7
#define SYS_LSEEK      8
#define SYS_BRK        9

/* External syscall helpers */
extern long syscall1(long nr, long a0);
extern long syscall3(long nr, long a0, long a1, long a2);

ssize_t write(int fd, const void *buf, size_t count) {
    return (ssize_t)syscall3(SYS_WRITE_FD, fd, (long)buf, count);
}

ssize_t read(int fd, void *buf, size_t count) {
    long ret = syscall3(SYS_READ, fd, (long)buf, count);
    if (ret == (long)-1) return -1;
    return (ssize_t)ret;
}

int open(const char *pathname) {
    long ret = syscall1(SYS_OPEN, (long)pathname);
    if (ret == (long)-1) return -1;
    return (int)ret;
}

int close(int fd) {
    long ret = syscall1(SYS_CLOSE, fd);
    return (ret == 0) ? 0 : -1;
}

off_t lseek(int fd, off_t offset, int whence) {
    long ret = syscall3(SYS_LSEEK, fd, offset, whence);
    if (ret == (long)-1) return -1;
    return (off_t)ret;
}

void _exit(int status) {
    syscall1(SYS_EXIT, status);
    while(1); /* Should not reach here */
}

int brk(void *addr) {
    long ret = syscall1(SYS_BRK, (long)addr);
    return (int)ret;
}

/* Simple sbrk implementation */
static void *current_brk = 0;

void *sbrk(long increment) {
    if (current_brk == 0) {
        /* Get current brk */
        current_brk = (void *)syscall1(SYS_BRK, 0);
    }
    
    if (increment == 0) {
        return current_brk;
    }
    
    void *old_brk = current_brk;
    void *new_brk = (void *)((char *)current_brk + increment);
    
    long ret = syscall1(SYS_BRK, (long)new_brk);
    if (ret == (long)new_brk) {
        current_brk = new_brk;
        return old_brk;
    }
    
    return (void *)-1;
}
