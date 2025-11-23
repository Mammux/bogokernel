#ifndef _CURSES_H
#define _CURSES_H

#include <stdarg.h> // IWYU pragma: keep
#include <stdbool.h>

/* Screen dimensions */
#define LINES 24
#define COLS 80

/* Character attributes */
#define A_NORMAL 0x00
#define A_STANDOUT 0x01
#define A_UNDERLINE 0x02
#define A_REVERSE 0x04
#define A_BLINK 0x08
#define A_BOLD 0x10
#define A_CHARTEXT 0xFF

/* ACS (Alternative Character Set) for line drawing */
#define ACS_ULCORNER '+' /* upper left corner */
#define ACS_LLCORNER '+' /* lower left corner */
#define ACS_URCORNER '+' /* upper right corner */
#define ACS_LRCORNER '+' /* lower right corner */
#define ACS_LTEE '+'     /* tee pointing right */
#define ACS_RTEE '+'     /* tee pointing left */
#define ACS_BTEE '+'     /* tee pointing up */
#define ACS_TTEE '+'     /* tee pointing down */
#define ACS_HLINE '-'    /* horizontal line */
#define ACS_VLINE '|'    /* vertical line */
#define ACS_PLUS '+'     /* large plus or crossover */
#define ACS_BULLET 'o'   /* bullet */
#define ACS_CKBOARD '#'  /* checker board (stipple) */
#define ACS_DEGREE 'o'   /* degree symbol */
#define ACS_PLMINUS '#'  /* plus/minus */
#define ACS_BOARD '#'    /* board of squares */
#define ACS_LANTERN '#'  /* lantern symbol */
#define ACS_BLOCK '#'    /* solid square block */

/* Special keys (for getch) */
#define KEY_DOWN 0402
#define KEY_UP 0403
#define KEY_LEFT 0404
#define KEY_RIGHT 0405
#define KEY_HOME 0406
#define KEY_PPAGE 0407
#define KEY_NPAGE 0410
#define KEY_END 0411
#define KEY_A1 0412
#define KEY_A3 0413
#define KEY_B2 0414
#define KEY_C1 0415
#define KEY_C3 0416

/* Error return value */
#define ERR (-1)

/* Boolean type for compatibility */
#ifndef TRUE
#define TRUE 1
#define FALSE 0
#endif

/* Character type */
typedef unsigned long chtype;

/* Window structure */
typedef struct _win_st {
  int _cury, _curx;          /* Current cursor position */
  int _maxy, _maxx;          /* Maximum coordinates */
  int _begy, _begx;          /* Screen coords of upper-left corner */
  short _flags;              /* Window state flags */
  chtype _attrs;             /* Current attributes */
  bool _clear;               /* Clear on next refresh */
  bool _leave;               /* Leave cursor after refresh */
  bool _scroll;              /* Scrolling allowed */
  bool _use_keypad;          /* Keypad mode */
  chtype **_y;               /* Pointer to line array */
  struct _win_st *_parent;   /* Parent window */
  struct _win_st *_children; /* First child window */
  struct _win_st *_sibling;  /* Next sibling window */
} WINDOW;

#define _IS_SUBWIN 0x01

/* Global variables */
extern WINDOW *stdscr; /* Standard screen */
extern WINDOW *curscr; /* Current screen state */

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

/* Macros for convenience */
#define mvinch(y, x) mvwinch(stdscr, y, x)
#define getyx(win, y, x) ((y) = (win)->_cury, (x) = (win)->_curx)

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
int wgetnstr(WINDOW *win, char *str, int n);
char *unctrl(chtype c);
chtype inch(void);
chtype winch(WINDOW *win);
int mvcur(int oldrow, int oldcol, int newrow, int newcol);
int keypad(WINDOW *win, bool bf);
char killchar(void);
char erasechar(void);
int flushinp(void);
int idlok(WINDOW *win, bool bf);
int baudrate(void);
int isendwin(void);
int halfdelay(int tenths);

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
WINDOW *subwin(WINDOW *parent, int nlines, int ncols, int begin_y, int begin_x);
int delwin(WINDOW *win);
int mvwin(WINDOW *win, int y, int x);
int touchwin(WINDOW *win);
int leaveok(WINDOW *win, bool bf);
int getmaxx(WINDOW *win);
int getmaxy(WINDOW *win);

/* Box and border drawing */
int box(WINDOW *win, chtype verch, chtype horch);
int border(chtype ls, chtype rs, chtype ts, chtype bs, chtype tl, chtype tr,
           chtype bl, chtype br);
int wborder(WINDOW *win, chtype ls, chtype rs, chtype ts, chtype bs, chtype tl,
            chtype tr, chtype bl, chtype br);
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
