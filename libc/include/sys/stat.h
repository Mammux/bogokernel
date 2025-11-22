#ifndef _SYS_STAT_H
#define _SYS_STAT_H

#include <sys/types.h>

/* File mode bits (simplified) */
#define S_IFMT   0170000  /* type of file */
#define S_IFREG  0100000  /* regular */
#define S_IFDIR  0040000  /* directory */

/* File status structure (minimal) */
struct stat {
    unsigned long st_dev;     /* Device */
    unsigned long st_ino;     /* Inode number */
    unsigned int  st_mode;    /* File type and mode */
    unsigned int  st_nlink;   /* Number of hard links */
    unsigned int  st_uid;     /* User ID of owner */
    unsigned int  st_gid;     /* Group ID of owner */
    unsigned long st_rdev;    /* Device ID (if special file) */
    long          st_size;    /* Total size, in bytes */
    long          st_atime;   /* Time of last access */
    long          st_mtime;   /* Time of last modification */
    long          st_ctime;   /* Time of last status change */
};

/* Get file status (stub) */
int stat(const char *pathname, struct stat *statbuf);

#endif /* _SYS_STAT_H */
