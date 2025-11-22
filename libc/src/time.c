/* libc/src/time.c - Time functions (stubs) */
#include <time.h>
#include <stddef.h>

/* time - get current time (stub) */
time_t time(time_t *tloc) {
    /* Return a fixed time value */
    time_t t = 0;
    if (tloc) {
        *tloc = t;
    }
    return t;
}

/* localtime - convert time to local time (stub) */
static struct tm _tm_stub = {
    .tm_sec = 0,
    .tm_min = 0,
    .tm_hour = 0,
    .tm_mday = 1,
    .tm_mon = 0,
    .tm_year = 124,  /* 2024 - 1900 */
    .tm_wday = 1,
    .tm_yday = 0,
    .tm_isdst = 0
};

struct tm *localtime(const time_t *timep) {
    (void)timep;
    return &_tm_stub;
}
