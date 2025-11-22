/* libc/src/stdlib.c - Standard library functions */
#include <stdlib.h>
#include <unistd.h>
#include <string.h>

void exit(int status) {
    _exit(status);
}

void abort(void) {
    _exit(1);
}

/* Simple bump allocator using sbrk */
void *malloc(size_t size) {
    if (size == 0) return 0;
    
    /* Align to 8 bytes */
    size = (size + 7) & ~7;
    
    void *ptr = sbrk(size);
    if (ptr == (void *)-1) return 0;
    
    return ptr;
}

void free(void *ptr) {
    /* Simple allocator doesn't support free */
    (void)ptr;
}

void *calloc(size_t nmemb, size_t size) {
    size_t total = nmemb * size;
    void *ptr = malloc(total);
    if (ptr) {
        memset(ptr, 0, total);
    }
    return ptr;
}

void *realloc(void *ptr, size_t size) {
    /* Simple implementation: allocate new, copy, don't free old */
    if (!ptr) return malloc(size);
    if (size == 0) {
        free(ptr);
        return 0;
    }
    
    void *new_ptr = malloc(size);
    if (new_ptr) {
        /* We don't know the old size, so this is unsafe */
        /* For a real implementation, we'd need to track allocation sizes */
        memcpy(new_ptr, ptr, size);
    }
    return new_ptr;
}

int atoi(const char *nptr) {
    int result = 0;
    int sign = 1;
    
    /* Skip whitespace */
    while (*nptr == ' ' || *nptr == '\t' || *nptr == '\n') nptr++;
    
    /* Handle sign */
    if (*nptr == '-') {
        sign = -1;
        nptr++;
    } else if (*nptr == '+') {
        nptr++;
    }
    
    /* Convert digits */
    while (*nptr >= '0' && *nptr <= '9') {
        result = result * 10 + (*nptr - '0');
        nptr++;
    }
    
    return sign * result;
}

long atol(const char *nptr) {
    return (long)atoi(nptr);
}

int abs(int j) {
    return j < 0 ? -j : j;
}