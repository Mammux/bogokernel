/* libc/src/signal.c - Signal handling stubs */
#include <signal.h>

/* signal - set signal handler (stub) */
sighandler_t signal(int signum, sighandler_t handler) {
    /* BogoKernel doesn't support signals */
    (void)signum;
    (void)handler;
    return SIG_DFL;
}
