/*
 * curses.c - Minimal curses implementation for BogoKernel
 *
 * Refactored to use per-window buffering and support subwindows.
 */

#include <curses.h>
#include <stdarg.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

/* Termcap compatibility - stub */
char *CE = "\033[K"; /* ANSI clear to end of line */

/* ANSI escape sequences */
#define ESC "\033"
#define CLEAR_SCREEN ESC "[2J" ESC "[H"
#define CURSOR_HOME ESC "[H"
#define CURSOR_HIDE ESC "[?25l"
#define CURSOR_SHOW ESC "[?25h"
#define ATTR_NORMAL ESC "[0m"
#define ATTR_BOLD ESC "[1m"
#define ATTR_REVERSE ESC "[7m"
#define CLEAR_EOL ESC "[K"

/* Global variables */
WINDOW *stdscr = NULL;
WINDOW *curscr = NULL;

static bool _echo = true;
static bool _cbreak = false;
static bool _nl = true;
static bool _initialized = false;

/* Helper: position cursor using ANSI escape */
static void _move_cursor(int y, int x) { printf(ESC "[%d;%dH", y + 1, x + 1); }

/* Helper: set attributes */
static void _set_attrs(chtype attrs) {
  if (attrs & A_STANDOUT || attrs & A_REVERSE) {
    printf(ATTR_REVERSE);
  } else if (attrs & A_BOLD) {
    printf(ATTR_BOLD);
  } else {
    printf(ATTR_NORMAL);
  }
}

/* Helper: allocate window buffer */
static int _alloc_win_buffer(WINDOW *win) {
  int i;
  win->_y = (chtype **)malloc(win->_maxy * sizeof(chtype *));
  if (!win->_y)
    return -1;

  for (i = 0; i < win->_maxy; i++) {
    win->_y[i] = (chtype *)malloc(win->_maxx * sizeof(chtype));
    if (!win->_y[i]) {
      /* Cleanup on failure */
      while (--i >= 0)
        free(win->_y[i]);
      free(win->_y);
      return -1;
    }
    /* Initialize with spaces */
    for (int j = 0; j < win->_maxx; j++) {
      win->_y[i][j] = ' ' | A_NORMAL;
    }
  }
  return 0;
}

/* Helper: free window buffer */
static void _free_win_buffer(WINDOW *win) {
  if (win->_y) {
    for (int i = 0; i < win->_maxy; i++) {
      free(win->_y[i]);
    }
    free(win->_y);
    win->_y = NULL;
  }
}

/* Initialize curses */
WINDOW *initscr(void) {
  if (_initialized) {
    return stdscr;
  }

  /* Initialize stdscr */
  stdscr = newwin(LINES, COLS, 0, 0);
  if (!stdscr)
    return NULL;

  /* Initialize curscr (represents physical screen state) */
  curscr = newwin(LINES, COLS, 0, 0);
  if (!curscr) {
    delwin(stdscr);
    return NULL;
  }

  /* Clear physical screen */
  printf(CLEAR_SCREEN);
  printf(CURSOR_HIDE);
  fflush(stdout);

  _initialized = true;
  return stdscr;
}

/* End curses mode */
int endwin(void) {
  if (!_initialized) {
    return -1;
  }

  /* Move cursor to bottom */
  _move_cursor(LINES - 1, 0);
  printf("\n");
  printf(ATTR_NORMAL);
  printf(CURSOR_SHOW);
  fflush(stdout);

  delwin(stdscr);
  delwin(curscr);
  stdscr = NULL;
  curscr = NULL;

  _initialized = false;
  return 0;
}

/* Move cursor in window */
int wmove(WINDOW *win, int y, int x) {
  if (!win || y < 0 || y >= win->_maxy || x < 0 || x >= win->_maxx) {
    return -1;
  }
  win->_cury = y;
  win->_curx = x;
  return 0;
}

int move(int y, int x) { return wmove(stdscr, y, x); }

/* Add character to window */
int waddch(WINDOW *win, chtype ch) {
  if (!win) {
    return -1;
  }

  int y = win->_cury;
  int x = win->_curx;

  if (y < 0 || y >= win->_maxy || x < 0 || x >= win->_maxx) {
    return -1;
  }

  char c = (char)(ch & A_CHARTEXT);
  chtype attrs = ch & ~A_CHARTEXT;

  /* Handle special characters */
  if (c == '\n') {
    win->_curx = 0;
    if (win->_cury < win->_maxy - 1) {
      win->_cury++;
    }
    return 0;
  } else if (c == '\r') {
    win->_curx = 0;
    return 0;
  } else if (c == '\t') {
    win->_curx = (win->_curx + 8) & ~7;
    if (win->_curx >= win->_maxx) {
      win->_curx = 0;
      if (win->_cury < win->_maxy - 1) {
        win->_cury++;
      }
    }
    return 0;
  }

  /* Store character in buffer */
  win->_y[y][x] = c | attrs | win->_attrs;

  /* Advance cursor */
  win->_curx++;
  if (win->_curx >= win->_maxx) {
    win->_curx = 0;
    if (win->_cury < win->_maxy - 1) {
      win->_cury++;
    }
  }

  return 0;
}

int addch(chtype ch) { return waddch(stdscr, ch); }

int mvwaddch(WINDOW *win, int y, int x, chtype ch) {
  if (wmove(win, y, x) == -1) {
    return -1;
  }
  return waddch(win, ch);
}

int mvaddch(int y, int x, chtype ch) { return mvwaddch(stdscr, y, x, ch); }

/* Add string to window */
int waddstr(WINDOW *win, const char *str) {
  if (!win || !str) {
    return -1;
  }
  while (*str) {
    if (waddch(win, *str++) == -1) {
      return -1;
    }
  }
  return 0;
}

int addstr(const char *str) { return waddstr(stdscr, str); }

int mvaddstr(int y, int x, const char *str) {
  if (move(y, x) == -1) {
    return -1;
  }
  return addstr(str);
}

/* Get character at position */
chtype mvwinch(WINDOW *win, int y, int x) {
  if (!win || y < 0 || y >= win->_maxy || x < 0 || x >= win->_maxx) {
    return (chtype)' ';
  }
  return win->_y[y][x];
}

/* Refresh screen */
int wrefresh(WINDOW *win) {
  if (!win || !_initialized) {
    return -1;
  }

  chtype last_attrs = A_NORMAL;
  bool force_redraw = win->_clear || (curscr && curscr->_clear);

  /* Calculate window bounds in screen coordinates */
  int win_top = win->_begy;
  int win_left = win->_begx;

  /* Iterate over window buffer */
  for (int y = 0; y < win->_maxy; y++) {
    int screen_y = win_top + y;
    if (screen_y >= LINES)
      break;

    for (int x = 0; x < win->_maxx; x++) {
      int screen_x = win_left + x;
      if (screen_x >= COLS)
        break;

      chtype ch = win->_y[y][x];

      /* Optimization: only update if different from physical screen */
      if (ch != curscr->_y[screen_y][screen_x] || force_redraw) {
        _move_cursor(screen_y, screen_x);

        chtype attrs = ch & ~A_CHARTEXT;
        if (attrs != last_attrs) {
          _set_attrs(attrs);
          last_attrs = attrs;
        }

        putchar((char)(ch & A_CHARTEXT));

        /* Update physical screen buffer */
        curscr->_y[screen_y][screen_x] = ch;
      }
    }
  }

  win->_clear = false;
  if (curscr)
    curscr->_clear = false;

  /* Reset attributes */
  if (last_attrs != A_NORMAL) {
    printf(ATTR_NORMAL);
  }

  /* Position cursor at window position */
  _move_cursor(win->_begy + win->_cury, win->_begx + win->_curx);

  fflush(stdout);
  return 0;
}

int refresh(void) { return wrefresh(stdscr); }

/* Clear window */
int wclear(WINDOW *win) {
  if (!win) {
    return -1;
  }

  /* Clear buffer */
  for (int y = 0; y < win->_maxy; y++) {
    for (int x = 0; x < win->_maxx; x++) {
      win->_y[y][x] = ' ' | A_NORMAL;
    }
  }

  win->_cury = 0;
  win->_curx = 0;
  win->_clear = true;

  return 0;
}

int clear(void) { return wclear(stdscr); }

int werase(WINDOW *win) { return wclear(win); }

int erase(void) { return clear(); }

/* Clear to end of line */
int wclrtoeol(WINDOW *win) {
  if (!win) {
    return -1;
  }

  int y = win->_cury;
  int x = win->_curx;

  for (; x < win->_maxx; x++) {
    win->_y[y][x] = ' ' | A_NORMAL;
  }

  return 0;
}

int clrtoeol(void) { return wclrtoeol(stdscr); }

/* Set clear flag */
int clearok(WINDOW *win, bool bf) {
  if (!win) {
    return -1;
  }
  win->_clear = bf;
  return 0;
}

/* Get character from input */
int wgetch(WINDOW *win) {
  char c;
  if (read(0, &c, 1) != 1) {
    return -1;
  }

  /* Echo if enabled */
  if (_echo && win) {
    waddch(win, c);
    wrefresh(win);
  }

  return (int)(unsigned char)c;
}

int getch(void) { return wgetch(stdscr); }

/* Terminal mode functions */
int cbreak(void) {
  _cbreak = true;
  return 0;
}

int nocbreak(void) {
  _cbreak = false;
  return 0;
}

int echo(void) {
  _echo = true;
  return 0;
}

int noecho(void) {
  _echo = false;
  return 0;
}

int nl(void) {
  _nl = true;
  return 0;
}

int nonl(void) {
  _nl = false;
  return 0;
}

int raw(void) {
  _cbreak = true;
  return 0;
}

int noraw(void) {
  _cbreak = false;
  return 0;
}

/* Attribute functions */
int wstandout(WINDOW *win) {
  if (!win) {
    return -1;
  }
  win->_attrs |= A_STANDOUT;
  return 0;
}

int standout(void) { return wstandout(stdscr); }

int wstandend(WINDOW *win) {
  if (!win) {
    return -1;
  }
  win->_attrs &= ~A_STANDOUT;
  return 0;
}

int standend(void) { return wstandend(stdscr); }

int attron(chtype attrs) {
  stdscr->_attrs |= attrs;
  return 0;
}

int attroff(chtype attrs) {
  stdscr->_attrs &= ~attrs;
  return 0;
}

int attrset(chtype attrs) {
  stdscr->_attrs = attrs;
  return 0;
}

/* Printf-style output */
int vwprintw(WINDOW *win, const char *fmt, va_list args) {
  char buf[256];
  vsnprintf(buf, sizeof(buf), fmt, args);
  return waddstr(win, buf);
}

int wprintw(WINDOW *win, const char *fmt, ...) {
  va_list args;
  va_start(args, fmt);
  int ret = vwprintw(win, fmt, args);
  va_end(args);
  return ret;
}

int printw(const char *fmt, ...) {
  va_list args;
  va_start(args, fmt);
  int ret = vwprintw(stdscr, fmt, args);
  va_end(args);
  return ret;
}

int mvwprintw(WINDOW *win, int y, int x, const char *fmt, ...) {
  if (wmove(win, y, x) == -1) {
    return -1;
  }
  va_list args;
  va_start(args, fmt);
  int ret = vwprintw(win, fmt, args);
  va_end(args);
  return ret;
}

int mvprintw(int y, int x, const char *fmt, ...) {
  if (move(y, x) == -1) {
    return -1;
  }
  va_list args;
  va_start(args, fmt);
  int ret = vwprintw(stdscr, fmt, args);
  va_end(args);
  return ret;
}

/* Window management */
WINDOW *newwin(int nlines, int ncols, int begin_y, int begin_x) {
  WINDOW *win = (WINDOW *)malloc(sizeof(WINDOW));
  if (!win) {
    return NULL;
  }

  win->_maxy = nlines;
  win->_maxx = ncols;
  win->_begy = begin_y;
  win->_begx = begin_x;
  win->_cury = 0;
  win->_curx = 0;
  win->_attrs = A_NORMAL;
  win->_clear = false;
  win->_leave = false;
  win->_scroll = false;
  win->_y = NULL;
  win->_flags = 0;
  win->_parent = NULL;
  win->_children = NULL;
  win->_sibling = NULL;

  if (_alloc_win_buffer(win) != 0) {
    free(win);
    return NULL;
  }

  return win;
}

int delwin(WINDOW *win) {
  if (!win || win == stdscr || win == curscr) {
    return -1;
  }

  /* Recursively delete children */
  while (win->_children) {
    delwin(win->_children);
  }

  /* Unlink from parent */
  if (win->_parent) {
    WINDOW *child = win->_parent->_children;
    if (child == win) {
      win->_parent->_children = win->_sibling;
    } else {
      while (child && child->_sibling != win) {
        child = child->_sibling;
      }
      if (child) {
        child->_sibling = win->_sibling;
      }
    }
  }

  /* Free memory */
  if (!(win->_flags & _IS_SUBWIN)) {
    _free_win_buffer(win);
  } else {
    /* For subwindows, we only free the pointer array, not the lines themselves
     */
    if (win->_y)
      free(win->_y);
  }

  free(win);
  return 0;
}

/* Create subwindow */
WINDOW *subwin(WINDOW *parent, int nlines, int ncols, int begin_y,
               int begin_x) {
  if (!parent) {
    return NULL;
  }

  /* Check bounds */
  if (begin_y < parent->_begy || begin_x < parent->_begx ||
      begin_y + nlines > parent->_begy + parent->_maxy ||
      begin_x + ncols > parent->_begx + parent->_maxx) {
    return NULL;
  }

  WINDOW *win = (WINDOW *)malloc(sizeof(WINDOW));
  if (!win)
    return NULL;

  win->_maxy = nlines;
  win->_maxx = ncols;
  win->_begy = begin_y;
  win->_begx = begin_x;
  win->_cury = 0;
  win->_curx = 0;
  win->_attrs = A_NORMAL;
  win->_clear = false;
  win->_leave = false;
  win->_scroll = false;
  win->_flags = _IS_SUBWIN;
  win->_parent = parent;
  win->_children = NULL;
  win->_sibling = parent->_children;
  parent->_children = win;

  /* Share memory with parent */
  win->_y = (chtype **)malloc(nlines * sizeof(chtype *));
  if (!win->_y) {
    /* Unlink and free */
    parent->_children = win->_sibling;
    free(win);
    return NULL;
  }

  int start_y = begin_y - parent->_begy;
  int start_x = begin_x - parent->_begx;

  for (int i = 0; i < nlines; i++) {
    win->_y[i] = &parent->_y[start_y + i][start_x];
  }

  return win;
}

/* Move window to new position */
int mvwin(WINDOW *win, int y, int x) {
  if (!win) {
    return -1;
  }
  win->_begy = y;
  win->_begx = x;
  return 0;
}

/* Mark window as changed (for refresh) */
int touchwin(WINDOW *win) {
  if (!win) {
    return -1;
  }
  win->_clear = true;
  return 0;
}

/* Control cursor leave behavior */
int leaveok(WINDOW *win, bool bf) {
  if (!win) {
    return -1;
  }
  win->_leave = bf;
  return 0;
}

/* Get window dimensions */
int getmaxx(WINDOW *win) {
  if (!win) {
    return -1;
  }
  return win->_maxx;
}

int getmaxy(WINDOW *win) {
  if (!win) {
    return -1;
  }
  return win->_maxy;
}

/* Additional curses stub functions for compatibility */

/* flushinp - flush input buffer */
int flushinp(void) {
  /* Stub - nothing to flush in our implementation */
  return 0;
}

/* idlok - enable/disable hardware insert/delete line */
int idlok(WINDOW *win, bool bf) {
  /* Stub - we don't support hardware scrolling */
  (void)win;
  (void)bf;
  return 0;
}

/* baudrate - get terminal baud rate */
int baudrate(void) {
  /* Return a reasonable default */
  return 9600;
}

/* isendwin - check if endwin has been called */
int isendwin(void) { return !_initialized; }

/* halfdelay - set half-delay mode */
int halfdelay(int tenths) {
  /* Stub - we don't implement timed input */
  (void)tenths;
  return 0;
}

/* Misc functions */
int beep(void) {
  putchar('\a');
  fflush(stdout);
  return 0;
}

int flash(void) {
  /* Visual bell - just beep for now */
  return beep();
}

/* Box and border drawing functions */
int wborder(WINDOW *win, chtype ls, chtype rs, chtype ts, chtype bs, chtype tl,
            chtype tr, chtype bl, chtype br) {
  if (!win) {
    return -1;
  }

  int y, x;
  int maxy = win->_maxy;
  int maxx = win->_maxx;

  /* Draw corners */
  if (tl)
    mvwaddch(win, 0, 0, tl);
  if (tr)
    mvwaddch(win, 0, maxx - 1, tr);
  if (bl)
    mvwaddch(win, maxy - 1, 0, bl);
  if (br)
    mvwaddch(win, maxy - 1, maxx - 1, br);

  /* Draw top and bottom borders */
  if (ts) {
    for (x = 1; x < maxx - 1; x++) {
      mvwaddch(win, 0, x, ts);
    }
  }
  if (bs) {
    for (x = 1; x < maxx - 1; x++) {
      mvwaddch(win, maxy - 1, x, bs);
    }
  }

  /* Draw left and right borders */
  if (ls) {
    for (y = 1; y < maxy - 1; y++) {
      mvwaddch(win, y, 0, ls);
    }
  }
  if (rs) {
    for (y = 1; y < maxy - 1; y++) {
      mvwaddch(win, y, maxx - 1, rs);
    }
  }

  return 0;
}

int border(chtype ls, chtype rs, chtype ts, chtype bs, chtype tl, chtype tr,
           chtype bl, chtype br) {
  return wborder(stdscr, ls, rs, ts, bs, tl, tr, bl, br);
}

int box(WINDOW *win, chtype verch, chtype horch) {
  if (!verch)
    verch = ACS_VLINE;
  if (!horch)
    horch = ACS_HLINE;
  return wborder(win, verch, verch, horch, horch, ACS_ULCORNER, ACS_URCORNER,
                 ACS_LLCORNER, ACS_LRCORNER);
}

int whline(WINDOW *win, chtype ch, int n) {
  if (!win || n < 0) {
    return -1;
  }

  if (!ch)
    ch = ACS_HLINE;

  int x = win->_curx;
  int y = win->_cury;

  for (int i = 0; i < n && x + i < win->_maxx; i++) {
    mvwaddch(win, y, x + i, ch);
  }

  return 0;
}

int hline(chtype ch, int n) { return whline(stdscr, ch, n); }

int wvline(WINDOW *win, chtype ch, int n) {
  if (!win || n < 0) {
    return -1;
  }

  if (!ch)
    ch = ACS_VLINE;

  int x = win->_curx;
  int y = win->_cury;

  for (int i = 0; i < n && y + i < win->_maxy; i++) {
    mvwaddch(win, y + i, x, ch);
  }

  return 0;
}

int vline(chtype ch, int n) { return wvline(stdscr, ch, n); }

int mvhline(int y, int x, chtype ch, int n) {
  if (move(y, x) == -1) {
    return -1;
  }
  return hline(ch, n);
}

int mvvline(int y, int x, chtype ch, int n) {
  if (move(y, x) == -1) {
    return -1;
  }
  return vline(ch, n);
}

int mvwhline(WINDOW *win, int y, int x, chtype ch, int n) {
  if (wmove(win, y, x) == -1) {
    return -1;
  }
  return whline(win, ch, n);
}

int mvwvline(WINDOW *win, int y, int x, chtype ch, int n) {
  if (wmove(win, y, x) == -1) {
    return -1;
  }
  return wvline(win, ch, n);
}

/* Missing functions implementation */

char *unctrl(chtype c) {
  static char buf[3];
  c &= 0x7F;
  if (c >= 0 && c < 32) {
    buf[0] = '^';
    buf[1] = c + '@';
    buf[2] = '\0';
  } else if (c == 127) {
    buf[0] = '^';
    buf[1] = '?';
    buf[2] = '\0';
  } else {
    buf[0] = (char)c;
    buf[1] = '\0';
  }
  return buf;
}

chtype winch(WINDOW *win) {
  if (!win)
    return (chtype)ERR;
  return mvwinch(win, win->_cury, win->_curx);
}

chtype inch(void) { return winch(stdscr); }

int keypad(WINDOW *win, bool bf) {
  if (!win)
    return ERR;
  win->_use_keypad = bf;
  return 0;
}

int mvcur(int oldrow, int oldcol, int newrow, int newcol) {
  /* Stub: just move the physical cursor */
  (void)oldrow;
  (void)oldcol;
  printf(ESC "[%d;%dH", newrow + 1, newcol + 1);
  return 0;
}

char erasechar(void) { return '\b'; }

char killchar(void) { return 0x15; /* Ctrl-U */ }

int wgetnstr(WINDOW *win, char *str, int n) {
  if (!win || !str || n < 1)
    return ERR;
  int i = 0;
  int ch;

  while (i < n - 1) {
    ch = wgetch(win);
    if (ch == ERR)
      return ERR;

    if (ch == '\n' || ch == '\r') {
      break;
    } else if (ch == '\b' || ch == 127) {
      if (i > 0) {
        i--;
        /* Simple visual backspace if echo is on */
        if (_echo) {
          waddstr(win, "\b \b");
          wrefresh(win);
        }
      }
    } else {
      str[i++] = (char)ch;
    }
  }
  str[i] = '\0';
  return 0;
}
