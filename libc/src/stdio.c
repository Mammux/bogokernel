/* libc/src/stdio.c - Basic I/O functions */
#include <stdio.h>
#include <unistd.h>
#include <string.h>

/* Fake FILE for stdout */
static FILE _stdout = { .fd = STDOUT_FILENO };
FILE *stdout = &_stdout;

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
    /* TODO: Implement sprintf */
    (void)str;
    (void)format;
    return 0;
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
