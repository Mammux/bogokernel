#ifndef _CTYPE_H
#define _CTYPE_H

/* Character classification functions */
int isalnum(int c);
int isalpha(int c);
int isdigit(int c);
int islower(int c);
int isupper(int c);
int isspace(int c);
int isprint(int c);

/* Character conversion functions */
int tolower(int c);
int toupper(int c);
int toascii(int c);

/* GNU extension - ctype table access (stub for compatibility) */
const unsigned short **__ctype_b_loc(void);

#endif /* _CTYPE_H */
