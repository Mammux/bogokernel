#include <curses.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

/* Termcap compatibility - stub */
char *CE = "\033[K";  /* ANSI clear to end of line */

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
    
    int y = win->_cury + win->_begy;
    int x = win->_curx + win->_begx;
    
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
    
    /* Calculate window bounds in screen coordinates */
    int win_top = win->_begy;
    int win_bottom = win->_begy + win->_maxy;
    int win_left = win->_begx;
    int win_right = win->_begx + win->_maxx;
    
    /* Clamp to screen bounds */
    if (win_top < 0) win_top = 0;
    if (win_bottom > LINES) win_bottom = LINES;
    if (win_left < 0) win_left = 0;
    if (win_right > COLS) win_right = COLS;
    
    /* 
     * CRITICAL: Update only dirty cells within THIS window's bounds.
     * We must NOT clear dirty flags outside the window area, as this would
     * interfere with other windows that have been marked for redraw.
     * Bug fix: Previously iterated over entire screen, causing inventory
     * overlay refreshes to clear dirty flags set by touchwin(stdscr).
     */
    for (int y = win_top; y < win_bottom; y++) {
        for (int x = win_left; x < win_right; x++) {
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

/* Box and border drawing functions */
int wborder(WINDOW *win, chtype ls, chtype rs, chtype ts, chtype bs,
            chtype tl, chtype tr, chtype bl, chtype br) {
    if (!win) {
        return -1;
    }
    
    int y, x;
    int maxy = win->_maxy;
    int maxx = win->_maxx;
    
    /* Draw corners */
    if (tl) mvwaddch(win, 0, 0, tl);
    if (tr) mvwaddch(win, 0, maxx - 1, tr);
    if (bl) mvwaddch(win, maxy - 1, 0, bl);
    if (br) mvwaddch(win, maxy - 1, maxx - 1, br);
    
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

int border(chtype ls, chtype rs, chtype ts, chtype bs,
           chtype tl, chtype tr, chtype bl, chtype br) {
    return wborder(stdscr, ls, rs, ts, bs, tl, tr, bl, br);
}

int box(WINDOW *win, chtype verch, chtype horch) {
    if (!verch) verch = ACS_VLINE;
    if (!horch) horch = ACS_HLINE;
    return wborder(win, verch, verch, horch, horch,
                   ACS_ULCORNER, ACS_URCORNER,
                   ACS_LLCORNER, ACS_LRCORNER);
}

int whline(WINDOW *win, chtype ch, int n) {
    if (!win || n < 0) {
        return -1;
    }
    
    if (!ch) ch = ACS_HLINE;
    
    int x = win->_curx;
    int y = win->_cury;
    
    for (int i = 0; i < n && x + i < win->_maxx; i++) {
        mvwaddch(win, y, x + i, ch);
    }
    
    return 0;
}

int hline(chtype ch, int n) {
    return whline(stdscr, ch, n);
}

int wvline(WINDOW *win, chtype ch, int n) {
    if (!win || n < 0) {
        return -1;
    }
    
    if (!ch) ch = ACS_VLINE;
    
    int x = win->_curx;
    int y = win->_cury;
    
    for (int i = 0; i < n && y + i < win->_maxy; i++) {
        mvwaddch(win, y + i, x, ch);
    }
    
    return 0;
}

int vline(chtype ch, int n) {
    return wvline(stdscr, ch, n);
}

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

/* Additional curses functions for rogue */

/* Get character at cursor position */
chtype inch(void) {
    return winch(stdscr);
}

chtype winch(WINDOW *win) {
    if (!win) {
        return (chtype)' ';
    }
    int y = win->_cury + win->_begy;
    int x = win->_curx + win->_begx;
    if (y < 0 || y >= LINES || x < 0 || x >= COLS) {
        return (chtype)' ';
    }
    return (chtype)screen_buffer[y][x] | attr_buffer[y][x];
}

/* Convert control character to printable string */
static char unctrl_buf[3];
char *unctrl(chtype c) {
    unsigned char ch = (unsigned char)(c & 0xFF);
    
    if (ch < 32) {
        /* Control character: ^A through ^Z, etc. */
        unctrl_buf[0] = '^';
        unctrl_buf[1] = ch + '@';
        unctrl_buf[2] = '\0';
    } else if (ch == 127) {
        /* DEL character */
        unctrl_buf[0] = '^';
        unctrl_buf[1] = '?';
        unctrl_buf[2] = '\0';
    } else {
        /* Printable character */
        unctrl_buf[0] = ch;
        unctrl_buf[1] = '\0';
    }
    
    return unctrl_buf;
}

/* Get string from window */
int wgetnstr(WINDOW *win, char *str, int n) {
    if (!win || !str || n <= 0) {
        return -1;
    }
    
    int i = 0;
    while (i < n - 1) {
        int ch = wgetch(win);
        if (ch == -1) {
            break;
        }
        if (ch == '\n' || ch == '\r') {
            break;
        }
        if (ch == 127 || ch == 8) {  /* Backspace or DEL */
            if (i > 0) {
                i--;
                /* Move cursor back and erase character */
                if (win->_curx > 0) {
                    win->_curx--;
                    waddch(win, ' ');
                    win->_curx--;
                }
            }
            continue;
        }
        str[i++] = (char)ch;
    }
    str[i] = '\0';
    return i;
}

/* Low-level cursor movement */
int mvcur(int oldrow, int oldcol, int newrow, int newcol) {
    /* In a real implementation, this would use terminfo capabilities */
    /* For now, just use ANSI escape sequences */
    (void)oldrow;
    (void)oldcol;
    printf(ESC "[%d;%dH", newrow + 1, newcol + 1);
    fflush(stdout);
    return 0;
}

/* Enable/disable keypad mode */
int keypad(WINDOW *win, bool bf) {
    if (!win) {
        return -1;
    }
    win->_use_keypad = bf;
    return 0;
}

/* Get kill character */
char killchar(void) {
    /* Return the default kill character (Ctrl-U) */
    return 0x15;  /* ^U */
}

/* Get erase character */
char erasechar(void) {
    /* Return the default erase character (Ctrl-H / backspace) */
    return 0x08;  /* ^H */
}

/* Create subwindow */
WINDOW *subwin(WINDOW *parent, int nlines, int ncols, int begin_y, int begin_x) {
    if (!parent) {
        return NULL;
    }
    
    /* For simplicity, create a regular window */
    /* In a full implementation, this would share the parent's buffer */
    return newwin(nlines, ncols, begin_y, begin_x);
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
    /* Mark all cells in the window as dirty */
    for (int y = win->_begy; y < win->_begy + win->_maxy && y < LINES; y++) {
        for (int x = win->_begx; x < win->_begx + win->_maxx && x < COLS; x++) {
            dirty[y][x] = true;
        }
    }
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
int isendwin(void) {
    return !_initialized;
}

/* halfdelay - set half-delay mode */
int halfdelay(int tenths) {
    /* Stub - we don't implement timed input */
    (void)tenths;
    return 0;
}



