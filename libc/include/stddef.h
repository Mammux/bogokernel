#ifndef _STDDEF_H
#define _STDDEF_H

/* NULL pointer constant */
#ifndef NULL
#define NULL ((void *)0)
#endif

/* size_t and ptrdiff_t are defined by the compiler */
#ifndef _SIZE_T
#define _SIZE_T
typedef __SIZE_TYPE__ size_t;
#endif

#ifndef _PTRDIFF_T
#define _PTRDIFF_T
typedef __PTRDIFF_TYPE__ ptrdiff_t;
#endif

/* offsetof macro */
#define offsetof(type, member) __builtin_offsetof(type, member)

#endif /* _STDDEF_H */
