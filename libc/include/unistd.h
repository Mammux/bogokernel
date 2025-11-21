#ifndef _UNISTD_H
#define _UNISTD_H

#include <sys/types.h>

/* Standard file descriptors */
#define STDIN_FILENO  0
#define STDOUT_FILENO 1
#define STDERR_FILENO 2

/* lseek whence values */
#define SEEK_SET 0
#define SEEK_CUR 1
#define SEEK_END 2

/* Function declarations */
ssize_t write(int fd, const void *buf, size_t count);
ssize_t read(int fd, void *buf, size_t count);
int open(const char *pathname);
int close(int fd);
off_t lseek(int fd, off_t offset, int whence);
void _exit(int status) __attribute__((noreturn));
int brk(void *addr);
void *sbrk(long increment);

#endif /* _UNISTD_H */
