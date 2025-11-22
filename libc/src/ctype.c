/* libc/src/ctype.c - Character classification and conversion */
#include <ctype.h>

/* Character classification functions */
int isalnum(int c) {
    return isalpha(c) || isdigit(c);
}

int isalpha(int c) {
    return (c >= 'A' && c <= 'Z') || (c >= 'a' && c <= 'z');
}

int isdigit(int c) {
    return (c >= '0' && c <= '9');
}

int islower(int c) {
    return (c >= 'a' && c <= 'z');
}

int isupper(int c) {
    return (c >= 'A' && c <= 'Z');
}

int isspace(int c) {
    return (c == ' ' || c == '\t' || c == '\n' || c == '\r' || c == '\f' || c == '\v');
}

int isprint(int c) {
    return (c >= 32 && c <= 126);
}

/* Character conversion functions */
int tolower(int c) {
    if (isupper(c)) {
        return c + ('a' - 'A');
    }
    return c;
}

int toupper(int c) {
    if (islower(c)) {
        return c - ('a' - 'A');
    }
    return c;
}

int toascii(int c) {
    return c & 0x7F;
}

/* GNU ctype extension - provide minimal stub */
/* This is used by some programs that check character types using bit tables */
static const unsigned short _ctype_table[384];  /* All zeros - minimal stub */
static const unsigned short *_ctype_ptr = _ctype_table + 128;

const unsigned short **__ctype_b_loc(void) {
    static const unsigned short *ptr = _ctype_table + 128;
    return &ptr;
}
