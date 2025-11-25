/* libc/src/syscall.c - Low-level syscall interface */

/* Syscall numbers matching kernel uapi */
#define SYS_WRITE      1
#define SYS_EXIT       2
#define SYS_WRITE_CSTR 3
#define SYS_OPEN       4
#define SYS_READ       5
#define SYS_WRITE_FD   6
#define SYS_CLOSE      7
#define SYS_LSEEK      8
#define SYS_BRK        9
#define SYS_GETTIME    10
#define SYS_POWEROFF   11
#define SYS_EXEC       12
#define SYS_EXECV      13
#define SYS_CREAT      14
#define SYS_UNLINK     15
#define SYS_STAT       16
#define SYS_CHMOD      17
#define SYS_READDIR    18
#define SYS_GET_FB_INFO 19
#define SYS_FB_FLUSH   20

/* Low-level syscall wrappers using inline assembly */

long syscall0(long nr) {
    long ret;
    __asm__ volatile (
        "mv a7, %1\n"
        "ecall\n"
        "mv %0, a0\n"
        : "=r"(ret)
        : "r"(nr)
        : "a0", "a7", "memory"
    );
    return ret;
}

long syscall1(long nr, long a0) {
    long ret;
    __asm__ volatile (
        "mv a7, %1\n"
        "mv a0, %2\n"
        "ecall\n"
        "mv %0, a0\n"
        : "=r"(ret)
        : "r"(nr), "r"(a0)
        : "a0", "a7", "memory"
    );
    return ret;
}

long syscall2(long nr, long a0, long a1) {
    long ret;
    __asm__ volatile (
        "mv a7, %1\n"
        "mv a0, %2\n"
        "mv a1, %3\n"
        "ecall\n"
        "mv %0, a0\n"
        : "=r"(ret)
        : "r"(nr), "r"(a0), "r"(a1)
        : "a0", "a1", "a7", "memory"
    );
    return ret;
}

long syscall3(long nr, long a0, long a1, long a2) {
    long ret;
    __asm__ volatile (
        "mv a7, %1\n"
        "mv a0, %2\n"
        "mv a1, %3\n"
        "mv a2, %4\n"
        "ecall\n"
        "mv %0, a0\n"
        : "=r"(ret)
        : "r"(nr), "r"(a0), "r"(a1), "r"(a2)
        : "a0", "a1", "a2", "a7", "memory"
    );
    return ret;
}

/* Framebuffer info structure (must match kernel side) */
struct fb_info {
    unsigned long width;
    unsigned long height;
    unsigned long stride;
    unsigned long addr;
};

/* Get framebuffer information - returns 0 on success, -1 on failure */
int get_fb_info(struct fb_info *info) {
    long ret = syscall1(SYS_GET_FB_INFO, (long)info);
    return (ret == (long)-1) ? -1 : 0;
}

/* Flush framebuffer to display - returns 0 on success, -1 on failure */
int fb_flush(void) {
    long ret = syscall0(SYS_FB_FLUSH);
    return (ret == (long)-1) ? -1 : 0;
}
