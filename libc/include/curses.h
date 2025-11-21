#ifndef _CURSES_H
#define _CURSES_H

#include <stdarg.h>
#include <stdbool.h>

/* Screen dimensions */
#define LINES 24
#define COLS 80

/* Character attributes */
#define A_NORMAL    0x00
#define A_STANDOUT  0x01
#define A_UNDERLINE 0x02
#define A_REVERSE   0x04
#define A_BLINK     0x08
#define A_BOLD      0x10
#define A_CHARTEXT  0xFF

/* ACS (Alternative Character Set) for line drawing */
#define ACS_ULCORNER '+'  /* upper left corner */
#define ACS_LLCORNER '+'  /* lower left corner */
#define ACS_URCORNER '+'  /* upper right corner */
#define ACS_LRCORNER '+'  /* lower right corner */
#define ACS_LTEE     '+'  /* tee pointing right */
#define ACS_RTEE     '+'  /* tee pointing left */
#define ACS_BTEE     '+'  /* tee pointing up */
#define ACS_TTEE     '+'  /* tee pointing down */
#define ACS_HLINE    '-'  /* horizontal line */
#define ACS_VLINE    '|'  /* vertical line */
#define ACS_PLUS     '+'  /* large plus or crossover */
#define ACS_BULLET   'o'  /* bullet */
#define ACS_CKBOARD  '#'  /* checker board (stipple) */
#define ACS_DEGREE   'o'  /* degree symbol */
#define ACS_PLMINUS  '#'  /* plus/minus */
#define ACS_BOARD    '#'  /* board of squares */
#define ACS_LANTERN  '#'  /* lantern symbol */
#define ACS_BLOCK    '#'  /* solid square block */

/* Special keys (for getch) */
#define KEY_DOWN    0402
#define KEY_UP      0403
#define KEY_LEFT    0404
#define KEY_RIGHT   0405

/* Boolean type for compatibility */
#ifndef TRUE
#define TRUE 1
#define FALSE 0
#endif

/* Character type */
typedef unsigned long chtype;

/* Window structure */
typedef struct _win_st {
    int _cury, _curx;           /* Current cursor position */
    int _maxy, _maxx;           /* Maximum coordinates */
    int _begy, _begx;           /* Screen coords of upper-left corner */
    short _flags;               /* Window state flags */
    chtype _attrs;              /* Current attributes */
    bool _clear;                /* Clear on next refresh */
    bool _leave;                /* Leave cursor after refresh */
    bool _scroll;               /* Scrolling allowed */
    bool _use_keypad;           /* Keypad mode */
    char **_y;                  /* Pointer to line array (not used in minimal) */
} WINDOW;

/* Global variables */
extern WINDOW *stdscr;          /* Standard screen */
extern WINDOW *curscr;          /* Current screen state */

/* Initialization and cleanup */
WINDOW *initscr(void);
int endwin(void);

/* Output functions */
int move(int y, int x);
int addch(chtype ch);
int mvaddch(int y, int x, chtype ch);
int addstr(const char *str);
int mvaddstr(int y, int x, const char *str);
int printw(const char *fmt, ...);
int mvprintw(int y, int x, const char *fmt, ...);
int wprintw(WINDOW *win, const char *fmt, ...);
int mvwprintw(WINDOW *win, int y, int x, const char *fmt, ...);

/* Window output functions */
int wmove(WINDOW *win, int y, int x);
int waddch(WINDOW *win, chtype ch);
int mvwaddch(WINDOW *win, int y, int x, chtype ch);
int waddstr(WINDOW *win, const char *str);
chtype mvwinch(WINDOW *win, int y, int x);

/* Screen update */
int refresh(void);
int wrefresh(WINDOW *win);
int clear(void);
int wclear(WINDOW *win);
int erase(void);
int werase(WINDOW *win);
int clrtoeol(void);
int wclrtoeol(WINDOW *win);
int clearok(WINDOW *win, bool bf);

/* Input functions */
int getch(void);
int wgetch(WINDOW *win);

/* Terminal mode functions */
int cbreak(void);
int nocbreak(void);
int echo(void);
int noecho(void);
int nl(void);
int nonl(void);
int raw(void);
int noraw(void);

/* Attribute functions */
int standout(void);
int standend(void);
int wstandout(WINDOW *win);
int wstandend(WINDOW *win);
int attron(chtype attrs);
int attroff(chtype attrs);
int attrset(chtype attrs);

/* Window management */
WINDOW *newwin(int nlines, int ncols, int begin_y, int begin_x);
int delwin(WINDOW *win);

/* Box and border drawing */
int box(WINDOW *win, chtype verch, chtype horch);
int border(chtype ls, chtype rs, chtype ts, chtype bs, 
           chtype tl, chtype tr, chtype bl, chtype br);
int wborder(WINDOW *win, chtype ls, chtype rs, chtype ts, chtype bs,
            chtype tl, chtype tr, chtype bl, chtype br);
int hline(chtype ch, int n);
int whline(WINDOW *win, chtype ch, int n);
int vline(chtype ch, int n);
int wvline(WINDOW *win, chtype ch, int n);
int mvhline(int y, int x, chtype ch, int n);
int mvvline(int y, int x, chtype ch, int n);
int mvwhline(WINDOW *win, int y, int x, chtype ch, int n);
int mvwvline(WINDOW *win, int y, int x, chtype ch, int n);

/* Misc functions */
int beep(void);
int flash(void);

#endif /* _CURSES_H */
