#ifndef _PWD_H
#define _PWD_H

#include <sys/types.h>

/* Password database structure (stub for compatibility) */
struct passwd {
    char *pw_name;      /* username */
    char *pw_passwd;    /* user password */
    uid_t pw_uid;       /* user ID */
    gid_t pw_gid;       /* group ID */
    char *pw_gecos;     /* user information */
    char *pw_dir;       /* home directory */
    char *pw_shell;     /* shell program */
};

/* Get password entry by UID (stub) */
struct passwd *getpwuid(uid_t uid);

#endif /* _PWD_H */
