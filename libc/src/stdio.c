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
    /* WARNING: This assumes str buffer is large enough (at least 4096 bytes) */
    /* For safety, callers should use vsnprintf with explicit size instead */
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
            
            /* Parse flags */
            int left_align = 0;
            int zero_pad = 0;
            int show_plus = 0;
            int space_prefix = 0;
            
            while (*format == '-' || *format == '0' || *format == '+' || *format == ' ') {
                if (*format == '-') left_align = 1;
                else if (*format == '0') zero_pad = 1;
                else if (*format == '+') show_plus = 1;
                else if (*format == ' ') space_prefix = 1;
                format++;
            }
            
            /* Parse width */
            int width = 0;
            if (*format == '*') {
                width = va_arg(args, int);
                if (width < 0) {
                    left_align = 1;
                    width = -width;
                }
                format++;
            } else {
                while (*format >= '0' && *format <= '9') {
                    width = width * 10 + (*format - '0');
                    format++;
                }
            }
            
            /* Parse precision (we'll ignore it for simplicity but need to consume args) */
            int precision = -1;
            if (*format == '.') {
                format++;
                precision = 0;
                if (*format == '*') {
                    precision = va_arg(args, int);
                    format++;
                } else {
                    while (*format >= '0' && *format <= '9') {
                        precision = precision * 10 + (*format - '0');
                        format++;
                    }
                }
            }
            
            /* Parse length modifiers (we'll skip them but advance past them) */
            while (*format == 'l' || *format == 'h' || *format == 'L' || *format == 'z') {
                format++;
            }
            
            /* Now handle the conversion specifier */
            switch (*format) {
                case 'd':
                case 'i': {
                    int val = va_arg(args, int);
                    int len = num_to_str(val, buf, 10, 1);
                    
                    /* Apply width and padding */
                    int pad_len = (width > len) ? width - len : 0;
                    if (!left_align && pad_len > 0) {
                        char pad_char = zero_pad ? '0' : ' ';
                        for (int i = 0; i < pad_len && count < (int)remaining; i++) {
                            str[count++] = pad_char;
                        }
                    }
                    
                    int copy_len = (len < (int)remaining - count) ? len : (int)remaining - count;
                    memcpy(str + count, buf, copy_len);
                    count += copy_len;
                    
                    if (left_align && pad_len > 0) {
                        for (int i = 0; i < pad_len && count < (int)remaining; i++) {
                            str[count++] = ' ';
                        }
                    }
                    break;
                }
                case 'u': {
                    unsigned int val = va_arg(args, unsigned int);
                    int len = num_to_str(val, buf, 10, 0);
                    
                    int pad_len = (width > len) ? width - len : 0;
                    if (!left_align && pad_len > 0) {
                        char pad_char = zero_pad ? '0' : ' ';
                        for (int i = 0; i < pad_len && count < (int)remaining; i++) {
                            str[count++] = pad_char;
                        }
                    }
                    
                    int copy_len = (len < (int)remaining - count) ? len : (int)remaining - count;
                    memcpy(str + count, buf, copy_len);
                    count += copy_len;
                    
                    if (left_align && pad_len > 0) {
                        for (int i = 0; i < pad_len && count < (int)remaining; i++) {
                            str[count++] = ' ';
                        }
                    }
                    break;
                }
                case 'x':
                case 'X': {
                    unsigned int val = va_arg(args, unsigned int);
                    int len = num_to_str(val, buf, 16, 0);
                    
                    int pad_len = (width > len) ? width - len : 0;
                    if (!left_align && pad_len > 0) {
                        char pad_char = zero_pad ? '0' : ' ';
                        for (int i = 0; i < pad_len && count < (int)remaining; i++) {
                            str[count++] = pad_char;
                        }
                    }
                    
                    int copy_len = (len < (int)remaining - count) ? len : (int)remaining - count;
                    memcpy(str + count, buf, copy_len);
                    count += copy_len;
                    
                    if (left_align && pad_len > 0) {
                        for (int i = 0; i < pad_len && count < (int)remaining; i++) {
                            str[count++] = ' ';
                        }
                    }
                    break;
                }
                case 's': {
                    const char *s = va_arg(args, const char *);
                    if (!s) s = "(null)";
                    int len = 0;
                    const char *tmp = s;
                    while (*tmp++) len++;
                    
                    int pad_len = (width > len) ? width - len : 0;
                    if (!left_align && pad_len > 0) {
                        for (int i = 0; i < pad_len && count < (int)remaining; i++) {
                            str[count++] = ' ';
                        }
                    }
                    
                    while (*s && count < (int)remaining) {
                        str[count++] = *s++;
                    }
                    
                    if (left_align && pad_len > 0) {
                        for (int i = 0; i < pad_len && count < (int)remaining; i++) {
                            str[count++] = ' ';
                        }
                    }
                    break;
                }
                case 'c': {
                    char c = (char)va_arg(args, int);
                    
                    int pad_len = width > 1 ? width - 1 : 0;
                    if (!left_align && pad_len > 0) {
                        for (int i = 0; i < pad_len && count < (int)remaining; i++) {
                            str[count++] = ' ';
                        }
                    }
                    
                    if (count < (int)remaining) {
                        str[count++] = c;
                    }
                    
                    if (left_align && pad_len > 0) {
                        for (int i = 0; i < pad_len && count < (int)remaining; i++) {
                            str[count++] = ' ';
                        }
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

/* File I/O functions - now with real filesystem support */

/* Track open FILE* for cleanup */
#define MAX_FILES 16
static FILE _file_table[MAX_FILES];
static int _file_used[MAX_FILES] = {0};

FILE *fopen(const char *pathname, const char *mode) {
    int fd = -1;
    int is_write = 0;
    
    if (!pathname || !mode) {
        errno = EINVAL;
        return NULL;
    }
    
    /* Parse mode - check for write modes */
    if (mode[0] == 'w' || mode[0] == 'a' || (mode[0] == 'r' && mode[1] == '+')) {
        is_write = 1;
    }
    
    /* Open or create file */
    if (mode[0] == 'w') {
        /* Write mode - create/truncate */
        fd = creat(pathname, 0644);
    } else if (mode[0] == 'r') {
        /* Read mode */
        fd = open(pathname);
    } else if (mode[0] == 'a') {
        /* Append mode - open existing or create */
        fd = open(pathname);
        if (fd < 0) {
            fd = creat(pathname, 0644);
        }
        /* Seek to end for append */
        if (fd >= 0) {
            lseek(fd, 0, SEEK_END);
        }
    }
    
    if (fd < 0) {
        errno = ENOENT;
        return NULL;
    }
    
    /* Find free slot in file table */
    for (int i = 0; i < MAX_FILES; i++) {
        if (!_file_used[i]) {
            _file_table[i].fd = fd;
            _file_used[i] = 1;
            return &_file_table[i];
        }
    }
    
    /* No free slots */
    close(fd);
    errno = EMFILE;
    return NULL;
}

int fclose(FILE *stream) {
    if (!stream) {
        errno = EBADF;
        return EOF;
    }
    
    /* Check if it's a special stream */
    if (stream == stdin || stream == stdout || stream == stderr) {
        return 0;
    }
    
    /* Find in file table and mark as free */
    for (int i = 0; i < MAX_FILES; i++) {
        if (_file_used[i] && &_file_table[i] == stream) {
            close(stream->fd);
            _file_used[i] = 0;
            return 0;
        }
    }
    
    errno = EBADF;
    return EOF;
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
    
    size_t total_bytes = size * nmemb;
    ssize_t n = read(stream->fd, ptr, total_bytes);
    
    if (n < 0) {
        errno = EIO;
        return 0;
    }
    
    return n / size;  /* Return number of complete objects read */
}

size_t fwrite(const void *ptr, size_t size, size_t nmemb, FILE *stream) {
    if (!ptr || !stream) {
        errno = EINVAL;
        return 0;
    }
    
    size_t total_bytes = size * nmemb;
    ssize_t n = write(stream->fd, ptr, total_bytes);
    
    if (n < 0) {
        errno = EIO;
        return 0;
    }
    
    return n / size;  /* Return number of complete objects written */
}

int putc(int c, FILE *stream) {
    if (!stream) {
        errno = EBADF;
        return EOF;
    }
    
    unsigned char ch = (unsigned char)c;
    ssize_t n = write(stream->fd, &ch, 1);
    
    if (n != 1) {
        errno = EIO;
        return EOF;
    }
    
    return (unsigned char)c;
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

/* Windows conio compatibility */
int _getch(void) {
    /* Read a single character without echo */
    char c;
    if (read(0, &c, 1) != 1) {
        return -1;
    }
    return (int)(unsigned char)c;
}



