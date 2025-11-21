#ifndef _STDLIB_H
#define _STDLIB_H

#include <sys/types.h>

/* Program termination */
void exit(int status) __attribute__((noreturn));
void abort(void) __attribute__((noreturn));

/* Memory allocation */
void *malloc(size_t size);
void free(void *ptr);
void *calloc(size_t nmemb, size_t size);
void *realloc(void *ptr, size_t size);

/* Numeric conversion */
int atoi(const char *nptr);
long atol(const char *nptr);

#endif /* _STDLIB_H */
