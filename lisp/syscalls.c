#include <stddef.h>

#define SYS_WRITE 6
#define SYS_EXIT 2

long syscall3(long nr, long arg0, long arg1, long arg2) {
    long ret;
    asm volatile (
        "mv a7, %1\n"
        "mv a0, %2\n"
        "mv a1, %3\n"
        "mv a2, %4\n"
        "ecall\n"
        "mv %0, a0\n"
        : "=r"(ret)
        : "r"(nr), "r"(arg0), "r"(arg1), "r"(arg2)
        : "a0", "a1", "a2", "a7", "memory"
    );
    return ret;
}

long syscall1(long nr, long arg0) {
    long ret;
    asm volatile (
        "mv a7, %1\n"
        "mv a0, %2\n"
        "ecall\n"
        "mv %0, a0\n"
        : "=r"(ret)
        : "r"(nr), "r"(arg0)
        : "a0", "a7", "memory"
    );
    return ret;
}

void exit(int code) {
    syscall1(SYS_EXIT, code);
    while(1);
}

long write(int fd, const void* buf, size_t count) {
    return syscall3(SYS_WRITE, fd, (long)buf, count);
}
