#ifndef _SIGNAL_H
#define _SIGNAL_H

/* Signal numbers (subset) */
#define SIGHUP    1   /* Hangup */
#define SIGINT    2   /* Interrupt */
#define SIGQUIT   3   /* Quit */
#define SIGILL    4   /* Illegal instruction */
#define SIGTRAP   5   /* Trace/breakpoint trap */
#define SIGABRT   6   /* Abort */
#define SIGBUS    7   /* Bus error */
#define SIGFPE    8   /* Floating point exception */
#define SIGKILL   9   /* Kill */
#define SIGSEGV   11  /* Segmentation fault */
#define SIGPIPE   13  /* Broken pipe */
#define SIGALRM   14  /* Alarm clock */
#define SIGTERM   15  /* Termination */

/* Signal handler type */
typedef void (*sighandler_t)(int);

/* Special signal handler values */
#define SIG_DFL  ((sighandler_t)0)   /* Default signal handling */
#define SIG_IGN  ((sighandler_t)1)   /* Ignore signal */
#define SIG_ERR  ((sighandler_t)-1)  /* Error return */

/* Set signal handler (stub) */
sighandler_t signal(int signum, sighandler_t handler);

#endif /* _SIGNAL_H */
