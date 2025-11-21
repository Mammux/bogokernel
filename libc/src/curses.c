#include <curses.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

/* ANSI escape sequences */
#define ESC "\033"
#define CLEAR_SCREEN    ESC "[2J" ESC "[H"
#define CURSOR_HOME     ESC "[H"
#define CURSOR_HIDE     ESC "[?25l"
#define CURSOR_SHOW     ESC "[?25h"
#define ATTR_NORMAL     ESC "[0m"
#define ATTR_BOLD       ESC "[1m"
#define ATTR_REVERSE    ESC "[7m"
#define CLEAR_EOL       ESC "[K"

/* Screen buffer */
static char screen_buffer[LINES][COLS];
static chtype attr_buffer[LINES][COLS];
static bool dirty[LINES][COLS];

/* Window structures */
static WINDOW _stdscr;
static WINDOW _curscr;
WINDOW *stdscr = &_stdscr;
WINDOW *curscr = &_curscr;

/* Terminal state */
static bool _echo = true;
static bool _cbreak = false;
static bool _nl = true;
static bool _initialized = false;

/* Helper: position cursor using ANSI escape */
static void _move_cursor(int y, int x) {
    printf(ESC "[%d;%dH", y + 1, x + 1);
}

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

/* Initialize curses */
WINDOW *initscr(void) {
    if (_initialized) {
        return stdscr;
    }
    
    /* Clear screen buffer */
    memset(screen_buffer, ' ', sizeof(screen_buffer));
    memset(attr_buffer, 0, sizeof(attr_buffer));
    memset(dirty, 1, sizeof(dirty));
    
    /* Initialize stdscr */
    stdscr->_cury = 0;
    stdscr->_curx = 0;
    stdscr->_maxy = LINES;
    stdscr->_maxx = COLS;
    stdscr->_begy = 0;
    stdscr->_begx = 0;
    stdscr->_attrs = A_NORMAL;
    stdscr->_clear = false;
    stdscr->_leave = false;
    stdscr->_scroll = false;
    
    /* Initialize curscr (same as stdscr) */
    memcpy(curscr, stdscr, sizeof(WINDOW));
    
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

int move(int y, int x) {
    return wmove(stdscr, y, x);
}

/* Add character to window */
int waddch(WINDOW *win, chtype ch) {
    if (!win) {
        return -1;
    }
    
    int y = win->_cury;
    int x = win->_curx;
    
    if (y < 0 || y >= LINES || x < 0 || x >= COLS) {
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
    screen_buffer[y][x] = c;
    attr_buffer[y][x] = attrs | win->_attrs;
    dirty[y][x] = true;
    
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

int addch(chtype ch) {
    return waddch(stdscr, ch);
}

int mvwaddch(WINDOW *win, int y, int x, chtype ch) {
    if (wmove(win, y, x) == -1) {
        return -1;
    }
    return waddch(win, ch);
}

int mvaddch(int y, int x, chtype ch) {
    return mvwaddch(stdscr, y, x, ch);
}

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

int addstr(const char *str) {
    return waddstr(stdscr, str);
}

int mvaddstr(int y, int x, const char *str) {
    if (move(y, x) == -1) {
        return -1;
    }
    return addstr(str);
}

/* Get character at position */
chtype mvwinch(WINDOW *win, int y, int x) {
    if (!win || y < 0 || y >= LINES || x < 0 || x >= COLS) {
        return (chtype)' ';
    }
    return (chtype)screen_buffer[y][x] | attr_buffer[y][x];
}

/* Refresh screen */
int wrefresh(WINDOW *win) {
    if (!win || !_initialized) {
        return -1;
    }
    
    chtype last_attrs = A_NORMAL;
    
    /* Update only dirty cells */
    for (int y = 0; y < LINES; y++) {
        for (int x = 0; x < COLS; x++) {
            if (dirty[y][x]) {
                /* Move cursor to position */
                _move_cursor(y, x);
                
                /* Set attributes if changed */
                chtype attrs = attr_buffer[y][x];
                if (attrs != last_attrs) {
                    _set_attrs(attrs);
                    last_attrs = attrs;
                }
                
                /* Output character */
                putchar(screen_buffer[y][x]);
                dirty[y][x] = false;
            }
        }
    }
    
    /* Reset attributes */
    if (last_attrs != A_NORMAL) {
        printf(ATTR_NORMAL);
    }
    
    /* Position cursor at window position */
    _move_cursor(win->_cury, win->_curx);
    
    fflush(stdout);
    return 0;
}

int refresh(void) {
    return wrefresh(stdscr);
}

/* Clear window */
int wclear(WINDOW *win) {
    if (!win) {
        return -1;
    }
    
    /* Clear buffer */
    for (int y = 0; y < LINES; y++) {
        for (int x = 0; x < COLS; x++) {
            screen_buffer[y][x] = ' ';
            attr_buffer[y][x] = A_NORMAL;
            dirty[y][x] = true;
        }
    }
    
    win->_cury = 0;
    win->_curx = 0;
    win->_clear = true;
    
    return 0;
}

int clear(void) {
    return wclear(stdscr);
}

int werase(WINDOW *win) {
    return wclear(win);
}

int erase(void) {
    return clear();
}

/* Clear to end of line */
int wclrtoeol(WINDOW *win) {
    if (!win) {
        return -1;
    }
    
    int y = win->_cury;
    int x = win->_curx;
    
    for (; x < COLS; x++) {
        screen_buffer[y][x] = ' ';
        attr_buffer[y][x] = A_NORMAL;
        dirty[y][x] = true;
    }
    
    return 0;
}

int clrtoeol(void) {
    return wclrtoeol(stdscr);
}

/* Set clear flag */
int clearok(WINDOW *win, bool bf) {
    if (!win) {
        return -1;
    }
    win->_clear = bf;
    if (bf) {
        /* Mark all cells as dirty */
        memset(dirty, 1, sizeof(dirty));
    }
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

int getch(void) {
    return wgetch(stdscr);
}

/* Terminal mode functions */
int cbreak(void) {
    _cbreak = true;
    /* In a real implementation, this would set terminal to cbreak mode */
    /* For BogoKernel, we assume input is already unbuffered */
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

int standout(void) {
    return wstandout(stdscr);
}

int wstandend(WINDOW *win) {
    if (!win) {
        return -1;
    }
    win->_attrs &= ~A_STANDOUT;
    return 0;
}

int standend(void) {
    return wstandend(stdscr);
}

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
    WINDOW *win = malloc(sizeof(WINDOW));
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
    
    return win;
}

int delwin(WINDOW *win) {
    if (!win || win == stdscr || win == curscr) {
        return -1;
    }
    free(win);
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
