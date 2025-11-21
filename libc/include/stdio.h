#ifndef _STDIO_H
#define _STDIO_H

#include <sys/types.h>
#include <stdarg.h>

#ifndef NULL
#define NULL ((void *)0)
#endif

/* File descriptor constants */
#define STDOUT_FILENO 1

/* Fake FILE structure for compatibility */
typedef struct {
    int fd;
} FILE;

extern FILE *stdout;

/* Basic I/O functions */
int putchar(int c);
int puts(const char *s);
int printf(const char *format, ...);
int sprintf(char *str, const char *format, ...);
int snprintf(char *str, size_t size, const char *format, ...);
int vsnprintf(char *str, size_t size, const char *format, va_list ap);
int fflush(FILE *stream);

#endif /* _STDIO_H */
