/* libc/src/stat.c - File status functions (stubs) */
#include <sys/stat.h>
#include <errno.h>

/* stat - get file status (stub) */
int stat(const char *pathname, struct stat *statbuf) {
    /* BogoKernel doesn't support stat */
    (void)pathname;
    (void)statbuf;
    errno = ENOENT;
    return -1;
}
