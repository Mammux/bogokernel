/* libc/src/stat.c - File status functions */
#include <sys/stat.h>
#include <errno.h>

/* External syscall wrappers */
extern long syscall2(long nr, long a0, long a1);

#define SYS_STAT  16
#define SYS_CHMOD 17

/* stat - get file status */
int stat(const char *pathname, struct stat *statbuf) {
    /* Call kernel stat syscall with simplified buffer */
    unsigned long stat_buf[2];  /* [0]=size, [1]=mode */
    long ret = syscall2(SYS_STAT, (long)pathname, (long)stat_buf);
    
    if (ret == -1UL) {  /* usize::MAX from kernel */
        errno = ENOENT;
        return -1;
    }
    
    /* Fill in the stat structure */
    if (statbuf) {
        statbuf->st_size = stat_buf[0];
        statbuf->st_mode = stat_buf[1];
        /* Other fields not supported yet */
        statbuf->st_dev = 0;
        statbuf->st_ino = 0;
        statbuf->st_nlink = 1;
        statbuf->st_uid = 0;
        statbuf->st_gid = 0;
        statbuf->st_rdev = 0;
        statbuf->st_blksize = 4096;
        statbuf->st_blocks = (stat_buf[0] + 4095) / 4096;
        statbuf->st_atime = 0;
        statbuf->st_mtime = 0;
        statbuf->st_ctime = 0;
    }
    
    return 0;
}
