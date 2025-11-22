#ifndef _STDIO_H
#define _STDIO_H

#include <sys/types.h>
#include <stdarg.h>

#ifndef NULL
#define NULL ((void *)0)
#endif

#ifndef EOF
#define EOF (-1)
#endif

#ifndef BUFSIZ
#define BUFSIZ 1024
#endif

/* File descriptor constants */
#define STDOUT_FILENO 1

/* Fake FILE structure for compatibility */
typedef struct {
    int fd;
} FILE;

extern FILE *stdout;
extern FILE *stdin;
extern FILE *stderr;

/* Basic I/O functions */
int putchar(int c);
int puts(const char *s);
int printf(const char *format, ...);
int sprintf(char *str, const char *format, ...);
int vsprintf(char *str, const char *format, va_list ap);
int snprintf(char *str, size_t size, const char *format, ...);
int vsnprintf(char *str, size_t size, const char *format, va_list ap);
int fflush(FILE *stream);

/* File I/O functions */
FILE *fopen(const char *pathname, const char *mode);
int fclose(FILE *stream);
char *fgets(char *s, int size, FILE *stream);
size_t fread(void *ptr, size_t size, size_t nmemb, FILE *stream);
int putc(int c, FILE *stream);
void rewind(FILE *stream);
int sscanf(const char *str, const char *format, ...);
void perror(const char *s);
int fputs(const char *s, FILE *stream);
int fputc(int c, FILE *stream);
void setbuf(FILE *stream, char *buf);

#endif /* _STDIO_H */
