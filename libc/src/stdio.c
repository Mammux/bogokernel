/* libc/src/stdio.c - Basic I/O functions */
#include <stdio.h>
#include <unistd.h>
#include <string.h>
#include <errno.h>

/* Global errno */
int errno = 0;

/* Fake FILE for stdout, stdin, and stderr */
static FILE _stdout = { .fd = STDOUT_FILENO };
static FILE _stdin = { .fd = 0 };
static FILE _stderr = { .fd = 2 };
FILE *stdout = &_stdout;
FILE *stdin = &_stdin;
FILE *stderr = &_stderr;

int putchar(int c) {
    char ch = (char)c;
    write(STDOUT_FILENO, &ch, 1);
    return c;
}

int puts(const char *s) {
    write(STDOUT_FILENO, s, strlen(s));
    putchar('\n');
    return 0;
}

/* Helper to convert number to string */
static int num_to_str(long num, char *buf, int base, int is_signed) {
    static const char digits[] = "0123456789abcdef";
    char tmp[32];
    int i = 0;
    int is_negative = 0;
    unsigned long n;
    
    if (is_signed && num < 0) {
        is_negative = 1;
        n = -num;
    } else {
        n = num;
    }
    
    if (n == 0) {
        tmp[i++] = '0';
    } else {
        while (n > 0) {
            tmp[i++] = digits[n % base];
            n /= base;
        }
    }
    
    int len = 0;
    if (is_negative) buf[len++] = '-';
    
    while (i > 0) {
        buf[len++] = tmp[--i];
    }
    buf[len] = '\0';
    
    return len;
}

/* Minimal printf implementation */
int printf(const char *format, ...) {
    va_list args;
    va_start(args, format);
    
    int count = 0;
    char buf[32];
    
    while (*format) {
        if (*format == '%') {
            format++;
            switch (*format) {
                case 'd':
                case 'i': {
                    int val = va_arg(args, int);
                    int len = num_to_str(val, buf, 10, 1);
                    write(STDOUT_FILENO, buf, len);
                    count += len;
                    break;
                }
                case 'u': {
                    unsigned int val = va_arg(args, unsigned int);
                    int len = num_to_str(val, buf, 10, 0);
                    write(STDOUT_FILENO, buf, len);
                    count += len;
                    break;
                }
                case 'x':
                case 'X': {
                    unsigned int val = va_arg(args, unsigned int);
                    int len = num_to_str(val, buf, 16, 0);
                    write(STDOUT_FILENO, buf, len);
                    count += len;
                    break;
                }
                case 's': {
                    const char *str = va_arg(args, const char *);
                    if (!str) str = "(null)";
                    int len = strlen(str);
                    write(STDOUT_FILENO, str, len);
                    count += len;
                    break;
                }
                case 'c': {
                    char c = (char)va_arg(args, int);
                    write(STDOUT_FILENO, &c, 1);
                    count++;
                    break;
                }
                case '%': {
                    write(STDOUT_FILENO, "%", 1);
                    count++;
                    break;
                }
                default:
                    write(STDOUT_FILENO, "%", 1);
                    write(STDOUT_FILENO, format, 1);
                    count += 2;
                    break;
            }
            format++;
        } else {
            write(STDOUT_FILENO, format, 1);
            count++;
            format++;
        }
    }
    
    va_end(args);
    return count;
}

int sprintf(char *str, const char *format, ...) {
    va_list args;
    va_start(args, format);
    int ret = vsprintf(str, format, args);
    va_end(args);
    return ret;
}

int vsprintf(char *str, const char *format, va_list args) {
    /* vsprintf is like vsnprintf with no size limit */
    /* For safety, we'll use a large buffer size */
    return vsnprintf(str, 4096, format, args);
}

int snprintf(char *str, size_t size, const char *format, ...) {
    va_list args;
    va_start(args, format);
    int ret = vsnprintf(str, size, format, args);
    va_end(args);
    return ret;
}

int vsnprintf(char *str, size_t size, const char *format, va_list args) {
    if (!str || size == 0) return 0;
    
    int count = 0;
    char buf[32];
    size_t remaining = size - 1;  /* Leave room for null terminator */
    
    while (*format && count < (int)remaining) {
        if (*format == '%') {
            format++;
            switch (*format) {
                case 'd':
                case 'i': {
                    int val = va_arg(args, int);
                    int len = num_to_str(val, buf, 10, 1);
                    int copy_len = (len < (int)remaining - count) ? len : (int)remaining - count;
                    memcpy(str + count, buf, copy_len);
                    count += copy_len;
                    break;
                }
                case 'u': {
                    unsigned int val = va_arg(args, unsigned int);
                    int len = num_to_str(val, buf, 10, 0);
                    int copy_len = (len < (int)remaining - count) ? len : (int)remaining - count;
                    memcpy(str + count, buf, copy_len);
                    count += copy_len;
                    break;
                }
                case 'x':
                case 'X': {
                    unsigned int val = va_arg(args, unsigned int);
                    int len = num_to_str(val, buf, 16, 0);
                    int copy_len = (len < (int)remaining - count) ? len : (int)remaining - count;
                    memcpy(str + count, buf, copy_len);
                    count += copy_len;
                    break;
                }
                case 's': {
                    const char *s = va_arg(args, const char *);
                    if (!s) s = "(null)";
                    while (*s && count < (int)remaining) {
                        str[count++] = *s++;
                    }
                    break;
                }
                case 'c': {
                    char c = (char)va_arg(args, int);
                    if (count < (int)remaining) {
                        str[count++] = c;
                    }
                    break;
                }
                case '%': {
                    if (count < (int)remaining) {
                        str[count++] = '%';
                    }
                    break;
                }
                default:
                    if (count < (int)remaining) {
                        str[count++] = '%';
                    }
                    if (count < (int)remaining && *format) {
                        str[count++] = *format;
                    }
                    break;
            }
            format++;
        } else {
            str[count++] = *format++;
        }
    }
    
    str[count] = '\0';
    return count;
}

int fflush(FILE *stream) {
    /* In our simple implementation, all writes are unbuffered */
    /* so fflush is a no-op */
    (void)stream;
    return 0;
}

/* File I/O functions - minimal stub implementations */
FILE *fopen(const char *pathname, const char *mode) {
    /* In BogoKernel, we don't have a real filesystem */
    /* These are stubs to allow compilation */
    (void)pathname;
    (void)mode;
    errno = ENOENT;
    return NULL;
}

int fclose(FILE *stream) {
    if (!stream) {
        errno = EBADF;
        return EOF;
    }
    /* Nothing to do in our stub implementation */
    return 0;
}

char *fgets(char *s, int size, FILE *stream) {
    if (!s || size <= 0 || !stream) {
        errno = EINVAL;
        return NULL;
    }
    
    if (stream == stdin) {
        /* Read from stdin */
        int i = 0;
        while (i < size - 1) {
            char c;
            ssize_t n = read(0, &c, 1);
            if (n <= 0) {
                break;
            }
            s[i++] = c;
            if (c == '\n') {
                break;
            }
        }
        s[i] = '\0';
        return (i > 0) ? s : NULL;
    }
    
    errno = EBADF;
    return NULL;
}

size_t fread(void *ptr, size_t size, size_t nmemb, FILE *stream) {
    if (!ptr || !stream) {
        errno = EINVAL;
        return 0;
    }
    
    /* Stub implementation */
    (void)size;
    (void)nmemb;
    errno = EBADF;
    return 0;
}

int putc(int c, FILE *stream) {
    if (!stream) {
        errno = EBADF;
        return EOF;
    }
    
    if (stream == stdout) {
        return putchar(c);
    }
    
    /* Stub for file output */
    errno = EBADF;
    return EOF;
}

void rewind(FILE *stream) {
    if (!stream) {
        return;
    }
    /* Stub - nothing to do */
}

/* Minimal sscanf implementation */
int sscanf(const char *str, const char *format, ...) {
    if (!str || !format) {
        return 0;
    }
    
    va_list args;
    va_start(args, format);
    
    int count = 0;
    const char *s = str;
    const char *f = format;
    
    while (*f) {
        /* Skip whitespace in format */
        while (*f == ' ' || *f == '\t' || *f == '\n') f++;
        
        if (*f == '%') {
            f++;
            if (*f == 'd') {
                /* Parse integer */
                int *ip = va_arg(args, int *);
                int sign = 1;
                int val = 0;
                
                /* Skip whitespace */
                while (*s == ' ' || *s == '\t') s++;
                
                /* Check for sign */
                if (*s == '-') {
                    sign = -1;
                    s++;
                } else if (*s == '+') {
                    s++;
                }
                
                /* Parse digits */
                if (*s >= '0' && *s <= '9') {
                    while (*s >= '0' && *s <= '9') {
                        val = val * 10 + (*s - '0');
                        s++;
                    }
                    *ip = sign * val;
                    count++;
                }
                f++;
            } else if (*f == 's') {
                /* Parse string */
                char *sp = va_arg(args, char *);
                
                /* Skip whitespace */
                while (*s == ' ' || *s == '\t') s++;
                
                /* Copy until whitespace */
                while (*s && *s != ' ' && *s != '\t' && *s != '\n') {
                    *sp++ = *s++;
                }
                *sp = '\0';
                count++;
                f++;
            } else {
                f++;
            }
        } else if (*f == *s) {
            f++;
            s++;
        } else {
            break;
        }
    }
    
    va_end(args);
    return count;
}

/* Print error message */
void perror(const char *s) {
    if (s && *s) {
        printf("%s: ", s);
    }
    printf("%s\n", strerror(errno));
}

/* fputs - write string to file stream */
int fputs(const char *s, FILE *stream) {
    if (!s || !stream) {
        errno = EINVAL;
        return EOF;
    }
    
    if (stream == stdout || stream == stderr) {
        size_t len = strlen(s);
        int fd = (stream == stdout) ? STDOUT_FILENO : 2;
        ssize_t written = write(fd, s, len);
        return (written == (ssize_t)len) ? 0 : EOF;
    }
    
    errno = EBADF;
    return EOF;
}

/* fputc - write character to file stream */
int fputc(int c, FILE *stream) {
    if (!stream) {
        errno = EBADF;
        return EOF;
    }
    
    if (stream == stdout) {
        return putchar(c);
    } else if (stream == stderr) {
        char ch = (char)c;
        write(2, &ch, 1);
        return c;
    }
    
    errno = EBADF;
    return EOF;
}

/* setbuf - set buffer for stream (stub) */
void setbuf(FILE *stream, char *buf) {
    /* Stub - our implementation is unbuffered */
    (void)stream;
    (void)buf;
}


