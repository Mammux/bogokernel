/*
 * curses.c - Minimal curses implementation for BogoKernel
 *
 * Refactored to use per-window buffering and support subwindows.
 * Supports GPU mode when compiled with -DGPU_ENABLED
 */

#include <curses.h>
#include <stdarg.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h> // IWYU pragma: keep
#include <unistd.h>

#ifdef GPU_ENABLED
#include <gpu.h>
#endif

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

#ifdef GPU_ENABLED
/*============================================================================
 * GPU MODE: Font data and framebuffer rendering
 *============================================================================*/

/* Font dimensions */
#define FONT_WIDTH 8
#define FONT_HEIGHT 16

/* GPU state */
static struct fb_info _fb_info;
static unsigned int *_framebuffer = NULL;
static bool _gpu_mode = false;

/* Colors (XRGB8888 format) */
#define COLOR_WHITE 0x00F0F0F0
#define COLOR_BLACK 0x00000000
#define COLOR_BRIGHT_WHITE 0x00FFFFFF

/*
 * 8x16 VGA-style bitmap font (ASCII 32-126)
 * Each character is represented by 16 bytes, one per row
 * Bit 0 (LSB) corresponds to the leftmost pixel in our rendering
 */
static const unsigned char FONT_8X16_NORMAL[95][16] = {
    // 32: Space
    {0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00},
    // 33: !
    {0x00, 0x00, 0x18, 0x3C, 0x3C, 0x3C, 0x18, 0x18, 0x18, 0x00, 0x18, 0x18, 0x00, 0x00, 0x00, 0x00},
    // 34: "
    {0x00, 0x66, 0x66, 0x66, 0x24, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00},
    // 35: #
    {0x00, 0x00, 0x00, 0x6C, 0x6C, 0xFE, 0x6C, 0x6C, 0x6C, 0xFE, 0x6C, 0x6C, 0x00, 0x00, 0x00, 0x00},
    // 36: $
    {0x18, 0x18, 0x7C, 0xC6, 0xC2, 0xC0, 0x7C, 0x06, 0x06, 0x86, 0xC6, 0x7C, 0x18, 0x18, 0x00, 0x00},
    // 37: %
    {0x00, 0x00, 0x00, 0x00, 0xC2, 0xC6, 0x0C, 0x18, 0x30, 0x60, 0xC6, 0x86, 0x00, 0x00, 0x00, 0x00},
    // 38: &
    {0x00, 0x00, 0x38, 0x6C, 0x6C, 0x38, 0x76, 0xDC, 0xCC, 0xCC, 0xCC, 0x76, 0x00, 0x00, 0x00, 0x00},
    // 39: '
    {0x00, 0x30, 0x30, 0x30, 0x60, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00},
    // 40: (
    {0x00, 0x00, 0x0C, 0x18, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x18, 0x0C, 0x00, 0x00, 0x00, 0x00},
    // 41: )
    {0x00, 0x00, 0x30, 0x18, 0x0C, 0x0C, 0x0C, 0x0C, 0x0C, 0x0C, 0x18, 0x30, 0x00, 0x00, 0x00, 0x00},
    // 42: *
    {0x00, 0x00, 0x00, 0x00, 0x00, 0x66, 0x3C, 0xFF, 0x3C, 0x66, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00},
    // 43: +
    {0x00, 0x00, 0x00, 0x00, 0x00, 0x18, 0x18, 0x7E, 0x18, 0x18, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00},
    // 44: ,
    {0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x18, 0x18, 0x18, 0x30, 0x00, 0x00, 0x00},
    // 45: -
    {0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xFE, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00},
    // 46: .
    {0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x18, 0x18, 0x00, 0x00, 0x00, 0x00},
    // 47: /
    {0x00, 0x00, 0x00, 0x00, 0x02, 0x06, 0x0C, 0x18, 0x30, 0x60, 0xC0, 0x80, 0x00, 0x00, 0x00, 0x00},
    // 48: 0
    {0x00, 0x00, 0x7C, 0xC6, 0xC6, 0xCE, 0xD6, 0xD6, 0xE6, 0xC6, 0xC6, 0x7C, 0x00, 0x00, 0x00, 0x00},
    // 49: 1
    {0x00, 0x00, 0x18, 0x38, 0x78, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x7E, 0x00, 0x00, 0x00, 0x00},
    // 50: 2
    {0x00, 0x00, 0x7C, 0xC6, 0x06, 0x0C, 0x18, 0x30, 0x60, 0xC0, 0xC6, 0xFE, 0x00, 0x00, 0x00, 0x00},
    // 51: 3
    {0x00, 0x00, 0x7C, 0xC6, 0x06, 0x06, 0x3C, 0x06, 0x06, 0x06, 0xC6, 0x7C, 0x00, 0x00, 0x00, 0x00},
    // 52: 4
    {0x00, 0x00, 0x0C, 0x1C, 0x3C, 0x6C, 0xCC, 0xFE, 0x0C, 0x0C, 0x0C, 0x1E, 0x00, 0x00, 0x00, 0x00},
    // 53: 5
    {0x00, 0x00, 0xFE, 0xC0, 0xC0, 0xC0, 0xFC, 0x06, 0x06, 0x06, 0xC6, 0x7C, 0x00, 0x00, 0x00, 0x00},
    // 54: 6
    {0x00, 0x00, 0x38, 0x60, 0xC0, 0xC0, 0xFC, 0xC6, 0xC6, 0xC6, 0xC6, 0x7C, 0x00, 0x00, 0x00, 0x00},
    // 55: 7
    {0x00, 0x00, 0xFE, 0xC6, 0x06, 0x06, 0x0C, 0x18, 0x30, 0x30, 0x30, 0x30, 0x00, 0x00, 0x00, 0x00},
    // 56: 8
    {0x00, 0x00, 0x7C, 0xC6, 0xC6, 0xC6, 0x7C, 0xC6, 0xC6, 0xC6, 0xC6, 0x7C, 0x00, 0x00, 0x00, 0x00},
    // 57: 9
    {0x00, 0x00, 0x7C, 0xC6, 0xC6, 0xC6, 0x7E, 0x06, 0x06, 0x06, 0x0C, 0x78, 0x00, 0x00, 0x00, 0x00},
    // 58: :
    {0x00, 0x00, 0x00, 0x00, 0x18, 0x18, 0x00, 0x00, 0x00, 0x18, 0x18, 0x00, 0x00, 0x00, 0x00, 0x00},
    // 59: ;
    {0x00, 0x00, 0x00, 0x00, 0x18, 0x18, 0x00, 0x00, 0x00, 0x18, 0x18, 0x30, 0x00, 0x00, 0x00, 0x00},
    // 60: <
    {0x00, 0x00, 0x00, 0x06, 0x0C, 0x18, 0x30, 0x60, 0x30, 0x18, 0x0C, 0x06, 0x00, 0x00, 0x00, 0x00},
    // 61: =
    {0x00, 0x00, 0x00, 0x00, 0x00, 0x7E, 0x00, 0x00, 0x7E, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00},
    // 62: >
    {0x00, 0x00, 0x00, 0x60, 0x30, 0x18, 0x0C, 0x06, 0x0C, 0x18, 0x30, 0x60, 0x00, 0x00, 0x00, 0x00},
    // 63: ?
    {0x00, 0x00, 0x7C, 0xC6, 0xC6, 0x0C, 0x18, 0x18, 0x18, 0x00, 0x18, 0x18, 0x00, 0x00, 0x00, 0x00},
    // 64: @
    {0x00, 0x00, 0x7C, 0xC6, 0xC6, 0xDE, 0xDE, 0xDE, 0xDC, 0xC0, 0xC0, 0x7C, 0x00, 0x00, 0x00, 0x00},
    // 65: A
    {0x00, 0x00, 0x10, 0x38, 0x6C, 0xC6, 0xC6, 0xFE, 0xC6, 0xC6, 0xC6, 0xC6, 0x00, 0x00, 0x00, 0x00},
    // 66: B
    {0x00, 0x00, 0xFC, 0x66, 0x66, 0x66, 0x7C, 0x66, 0x66, 0x66, 0x66, 0xFC, 0x00, 0x00, 0x00, 0x00},
    // 67: C
    {0x00, 0x00, 0x3C, 0x66, 0xC2, 0xC0, 0xC0, 0xC0, 0xC0, 0xC2, 0x66, 0x3C, 0x00, 0x00, 0x00, 0x00},
    // 68: D
    {0x00, 0x00, 0xF8, 0x6C, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x6C, 0xF8, 0x00, 0x00, 0x00, 0x00},
    // 69: E
    {0x00, 0x00, 0xFE, 0x66, 0x62, 0x68, 0x78, 0x68, 0x60, 0x62, 0x66, 0xFE, 0x00, 0x00, 0x00, 0x00},
    // 70: F
    {0x00, 0x00, 0xFE, 0x66, 0x62, 0x68, 0x78, 0x68, 0x60, 0x60, 0x60, 0xF0, 0x00, 0x00, 0x00, 0x00},
    // 71: G
    {0x00, 0x00, 0x3C, 0x66, 0xC2, 0xC0, 0xC0, 0xDE, 0xC6, 0xC6, 0x66, 0x3A, 0x00, 0x00, 0x00, 0x00},
    // 72: H
    {0x00, 0x00, 0xC6, 0xC6, 0xC6, 0xC6, 0xFE, 0xC6, 0xC6, 0xC6, 0xC6, 0xC6, 0x00, 0x00, 0x00, 0x00},
    // 73: I
    {0x00, 0x00, 0x3C, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x3C, 0x00, 0x00, 0x00, 0x00},
    // 74: J
    {0x00, 0x00, 0x1E, 0x0C, 0x0C, 0x0C, 0x0C, 0x0C, 0xCC, 0xCC, 0xCC, 0x78, 0x00, 0x00, 0x00, 0x00},
    // 75: K
    {0x00, 0x00, 0xE6, 0x66, 0x6C, 0x6C, 0x78, 0x78, 0x6C, 0x66, 0x66, 0xE6, 0x00, 0x00, 0x00, 0x00},
    // 76: L
    {0x00, 0x00, 0xF0, 0x60, 0x60, 0x60, 0x60, 0x60, 0x60, 0x62, 0x66, 0xFE, 0x00, 0x00, 0x00, 0x00},
    // 77: M
    {0x00, 0x00, 0xC6, 0xEE, 0xFE, 0xFE, 0xD6, 0xC6, 0xC6, 0xC6, 0xC6, 0xC6, 0x00, 0x00, 0x00, 0x00},
    // 78: N
    {0x00, 0x00, 0xC6, 0xE6, 0xF6, 0xFE, 0xDE, 0xCE, 0xC6, 0xC6, 0xC6, 0xC6, 0x00, 0x00, 0x00, 0x00},
    // 79: O
    {0x00, 0x00, 0x38, 0x6C, 0xC6, 0xC6, 0xC6, 0xC6, 0xC6, 0xC6, 0x6C, 0x38, 0x00, 0x00, 0x00, 0x00},
    // 80: P
    {0x00, 0x00, 0xFC, 0x66, 0x66, 0x66, 0x7C, 0x60, 0x60, 0x60, 0x60, 0xF0, 0x00, 0x00, 0x00, 0x00},
    // 81: Q
    {0x00, 0x00, 0x7C, 0xC6, 0xC6, 0xC6, 0xC6, 0xC6, 0xC6, 0xD6, 0xDE, 0x7C, 0x0C, 0x0E, 0x00, 0x00},
    // 82: R
    {0x00, 0x00, 0xFC, 0x66, 0x66, 0x66, 0x7C, 0x6C, 0x66, 0x66, 0x66, 0xE6, 0x00, 0x00, 0x00, 0x00},
    // 83: S
    {0x00, 0x00, 0x7C, 0xC6, 0xC6, 0x60, 0x38, 0x0C, 0x06, 0xC6, 0xC6, 0x7C, 0x00, 0x00, 0x00, 0x00},
    // 84: T
    {0x00, 0x00, 0x7E, 0x7E, 0x5A, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x3C, 0x00, 0x00, 0x00, 0x00},
    // 85: U
    {0x00, 0x00, 0xC6, 0xC6, 0xC6, 0xC6, 0xC6, 0xC6, 0xC6, 0xC6, 0xC6, 0x7C, 0x00, 0x00, 0x00, 0x00},
    // 86: V
    {0x00, 0x00, 0xC6, 0xC6, 0xC6, 0xC6, 0xC6, 0xC6, 0xC6, 0x6C, 0x38, 0x10, 0x00, 0x00, 0x00, 0x00},
    // 87: W
    {0x00, 0x00, 0xC6, 0xC6, 0xC6, 0xC6, 0xD6, 0xD6, 0xD6, 0xFE, 0x6C, 0x6C, 0x00, 0x00, 0x00, 0x00},
    // 88: X
    {0x00, 0x00, 0xC6, 0xC6, 0x6C, 0x7C, 0x38, 0x38, 0x7C, 0x6C, 0xC6, 0xC6, 0x00, 0x00, 0x00, 0x00},
    // 89: Y
    {0x00, 0x00, 0x66, 0x66, 0x66, 0x66, 0x3C, 0x18, 0x18, 0x18, 0x18, 0x3C, 0x00, 0x00, 0x00, 0x00},
    // 90: Z
    {0x00, 0x00, 0xFE, 0xC6, 0x86, 0x0C, 0x18, 0x30, 0x60, 0xC2, 0xC6, 0xFE, 0x00, 0x00, 0x00, 0x00},
    // 91: [
    {0x00, 0x00, 0x3C, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x3C, 0x00, 0x00, 0x00, 0x00},
    // 92: \
    {0x00, 0x00, 0x00, 0x80, 0xC0, 0xE0, 0x70, 0x38, 0x1C, 0x0E, 0x06, 0x02, 0x00, 0x00, 0x00, 0x00},
    // 93: ]
    {0x00, 0x00, 0x3C, 0x0C, 0x0C, 0x0C, 0x0C, 0x0C, 0x0C, 0x0C, 0x0C, 0x3C, 0x00, 0x00, 0x00, 0x00},
    // 94: ^
    {0x10, 0x38, 0x6C, 0xC6, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00},
    // 95: _
    {0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xFF, 0x00, 0x00},
    // 96: `
    {0x00, 0x30, 0x18, 0x0C, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00},
    // 97: a
    {0x00, 0x00, 0x00, 0x00, 0x00, 0x78, 0x0C, 0x7C, 0xCC, 0xCC, 0xCC, 0x76, 0x00, 0x00, 0x00, 0x00},
    // 98: b
    {0x00, 0x00, 0xE0, 0x60, 0x60, 0x78, 0x6C, 0x66, 0x66, 0x66, 0x66, 0x7C, 0x00, 0x00, 0x00, 0x00},
    // 99: c
    {0x00, 0x00, 0x00, 0x00, 0x00, 0x7C, 0xC6, 0xC0, 0xC0, 0xC0, 0xC6, 0x7C, 0x00, 0x00, 0x00, 0x00},
    // 100: d
    {0x00, 0x00, 0x1C, 0x0C, 0x0C, 0x3C, 0x6C, 0xCC, 0xCC, 0xCC, 0xCC, 0x76, 0x00, 0x00, 0x00, 0x00},
    // 101: e
    {0x00, 0x00, 0x00, 0x00, 0x00, 0x7C, 0xC6, 0xFE, 0xC0, 0xC0, 0xC6, 0x7C, 0x00, 0x00, 0x00, 0x00},
    // 102: f
    {0x00, 0x00, 0x38, 0x6C, 0x64, 0x60, 0xF0, 0x60, 0x60, 0x60, 0x60, 0xF0, 0x00, 0x00, 0x00, 0x00},
    // 103: g
    {0x00, 0x00, 0x00, 0x00, 0x00, 0x76, 0xCC, 0xCC, 0xCC, 0xCC, 0xCC, 0x7C, 0x0C, 0xCC, 0x78, 0x00},
    // 104: h
    {0x00, 0x00, 0xE0, 0x60, 0x60, 0x6C, 0x76, 0x66, 0x66, 0x66, 0x66, 0xE6, 0x00, 0x00, 0x00, 0x00},
    // 105: i
    {0x00, 0x00, 0x18, 0x18, 0x00, 0x38, 0x18, 0x18, 0x18, 0x18, 0x18, 0x3C, 0x00, 0x00, 0x00, 0x00},
    // 106: j
    {0x00, 0x00, 0x06, 0x06, 0x00, 0x0E, 0x06, 0x06, 0x06, 0x06, 0x06, 0x06, 0x66, 0x66, 0x3C, 0x00},
    // 107: k
    {0x00, 0x00, 0xE0, 0x60, 0x60, 0x66, 0x6C, 0x78, 0x78, 0x6C, 0x66, 0xE6, 0x00, 0x00, 0x00, 0x00},
    // 108: l
    {0x00, 0x00, 0x38, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x3C, 0x00, 0x00, 0x00, 0x00},
    // 109: m
    {0x00, 0x00, 0x00, 0x00, 0x00, 0xEC, 0xFE, 0xD6, 0xD6, 0xD6, 0xD6, 0xC6, 0x00, 0x00, 0x00, 0x00},
    // 110: n
    {0x00, 0x00, 0x00, 0x00, 0x00, 0xDC, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x00, 0x00, 0x00, 0x00},
    // 111: o
    {0x00, 0x00, 0x00, 0x00, 0x00, 0x7C, 0xC6, 0xC6, 0xC6, 0xC6, 0xC6, 0x7C, 0x00, 0x00, 0x00, 0x00},
    // 112: p
    {0x00, 0x00, 0x00, 0x00, 0x00, 0xDC, 0x66, 0x66, 0x66, 0x66, 0x66, 0x7C, 0x60, 0x60, 0xF0, 0x00},
    // 113: q
    {0x00, 0x00, 0x00, 0x00, 0x00, 0x76, 0xCC, 0xCC, 0xCC, 0xCC, 0xCC, 0x7C, 0x0C, 0x0C, 0x1E, 0x00},
    // 114: r
    {0x00, 0x00, 0x00, 0x00, 0x00, 0xDC, 0x76, 0x66, 0x60, 0x60, 0x60, 0xF0, 0x00, 0x00, 0x00, 0x00},
    // 115: s
    {0x00, 0x00, 0x00, 0x00, 0x00, 0x7C, 0xC6, 0x60, 0x38, 0x0C, 0xC6, 0x7C, 0x00, 0x00, 0x00, 0x00},
    // 116: t
    {0x00, 0x00, 0x10, 0x30, 0x30, 0xFC, 0x30, 0x30, 0x30, 0x30, 0x36, 0x1C, 0x00, 0x00, 0x00, 0x00},
    // 117: u
    {0x00, 0x00, 0x00, 0x00, 0x00, 0xCC, 0xCC, 0xCC, 0xCC, 0xCC, 0xCC, 0x76, 0x00, 0x00, 0x00, 0x00},
    // 118: v
    {0x00, 0x00, 0x00, 0x00, 0x00, 0xC6, 0xC6, 0xC6, 0xC6, 0xC6, 0x6C, 0x38, 0x00, 0x00, 0x00, 0x00},
    // 119: w
    {0x00, 0x00, 0x00, 0x00, 0x00, 0xC6, 0xC6, 0xD6, 0xD6, 0xD6, 0xFE, 0x6C, 0x00, 0x00, 0x00, 0x00},
    // 120: x
    {0x00, 0x00, 0x00, 0x00, 0x00, 0xC6, 0x6C, 0x38, 0x38, 0x38, 0x6C, 0xC6, 0x00, 0x00, 0x00, 0x00},
    // 121: y
    {0x00, 0x00, 0x00, 0x00, 0x00, 0xC6, 0xC6, 0xC6, 0xC6, 0xC6, 0xC6, 0x7E, 0x06, 0x0C, 0xF8, 0x00},
    // 122: z
    {0x00, 0x00, 0x00, 0x00, 0x00, 0xFE, 0xCC, 0x18, 0x30, 0x60, 0xC6, 0xFE, 0x00, 0x00, 0x00, 0x00},
    // 123: {
    {0x00, 0x00, 0x0E, 0x18, 0x18, 0x18, 0x70, 0x18, 0x18, 0x18, 0x18, 0x0E, 0x00, 0x00, 0x00, 0x00},
    // 124: |
    {0x00, 0x00, 0x18, 0x18, 0x18, 0x18, 0x00, 0x18, 0x18, 0x18, 0x18, 0x18, 0x00, 0x00, 0x00, 0x00},
    // 125: }
    {0x00, 0x00, 0x70, 0x18, 0x18, 0x18, 0x0E, 0x18, 0x18, 0x18, 0x18, 0x70, 0x00, 0x00, 0x00, 0x00},
    // 126: ~
    {0x00, 0x00, 0x76, 0xDC, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00},
};

/* Get font bitmap for a character (currently only normal 8x16 font supported) */
static const unsigned char *_gpu_get_font(unsigned char c, bool bold) {
  /* Bold not yet implemented for 8x16 font, use normal */
  (void)bold;
  
  if (c < 32 || c > 126) {
    return FONT_8X16_NORMAL[0]; /* Return space */
  }
  return FONT_8X16_NORMAL[c - 32];
}

/* Draw a character directly to the framebuffer */
static void _gpu_draw_char(int screen_x, int screen_y, unsigned char c,
                           chtype attrs) {
  if (!_framebuffer)
    return;

  bool bold = (attrs & A_BOLD) != 0;
  bool reverse = (attrs & (A_STANDOUT | A_REVERSE)) != 0;

  const unsigned char *bitmap = _gpu_get_font(c, bold);

  unsigned int fg = reverse ? COLOR_BLACK : COLOR_WHITE;
  unsigned int bg = reverse ? COLOR_WHITE : COLOR_BLACK;

  /* Use brighter white for bold text */
  if (bold && !reverse) {
    fg = COLOR_BRIGHT_WHITE;
  }

  int pixel_x = screen_x * FONT_WIDTH;
  int pixel_y = screen_y * FONT_HEIGHT;

  unsigned long fb_width = _fb_info.width;
  unsigned long fb_height = _fb_info.height;
  unsigned long fb_size = fb_width * fb_height;

  for (int row = 0; row < FONT_HEIGHT; row++) {
    unsigned char bitmap_row = bitmap[row];
    int y = pixel_y + row;

    if ((unsigned long)y >= fb_height)
      break;

    for (int col = 0; col < FONT_WIDTH; col++) {
      int x = pixel_x + col;

      if ((unsigned long)x >= fb_width)
        break;

      /* Calculate offset with bounds check */
      unsigned long offset = (unsigned long)y * fb_width + (unsigned long)x;
      if (offset >= fb_size)
        continue; /* Safety bounds check */

      /* Check if pixel is set (bit 0 is leftmost in our rendering) */
      bool pixel_set = (bitmap_row & (1 << col)) != 0;
      unsigned int color = pixel_set ? fg : bg;

      _framebuffer[offset] = color;
    }
  }
}

/* Clear the framebuffer screen */
static void _gpu_clear_screen(void) {
  if (!_framebuffer)
    return;

  unsigned long total_pixels = _fb_info.width * _fb_info.height;
  for (unsigned long i = 0; i < total_pixels; i++) {
    _framebuffer[i] = COLOR_BLACK;
  }
}

/* Initialize GPU mode */
static bool _gpu_init(void) {
  if (get_fb_info(&_fb_info) != 0) {
    return false;
  }

  /* Validate framebuffer address - must be non-zero and reasonably sized */
  if (_fb_info.addr == 0 || _fb_info.width == 0 || _fb_info.height == 0) {
    return false;
  }

  _framebuffer = (unsigned int *)_fb_info.addr;

  _gpu_mode = true;
  _gpu_clear_screen();
  fb_flush();
  return true;
}

/* GPU mode refresh - render entire window to framebuffer */
static int _gpu_wrefresh(WINDOW *win) {
  if (!_gpu_mode || !_framebuffer)
    return -1;

  int win_top = win->_begy;
  int win_left = win->_begx;

  bool force_redraw = win->_clear || (curscr && curscr->_clear);

  for (int y = 0; y < win->_maxy; y++) {
    int screen_y = win_top + y;
    if (screen_y >= LINES)
      break;

    for (int x = 0; x < win->_maxx; x++) {
      int screen_x = win_left + x;
      if (screen_x >= COLS)
        break;

      chtype ch = win->_y[y][x];

      /* Only update if different from physical screen or force redraw */
      if (ch != curscr->_y[screen_y][screen_x] || force_redraw) {
        unsigned char c = (unsigned char)(ch & A_CHARTEXT);
        chtype attrs = ch & ~A_CHARTEXT;

        _gpu_draw_char(screen_x, screen_y, c, attrs);

        curscr->_y[screen_y][screen_x] = ch;
      }
    }
  }

  win->_clear = false;
  if (curscr)
    curscr->_clear = false;

  /* Flush framebuffer to display */
  fb_flush();

  return 0;
}

#endif /* GPU_ENABLED */

/*============================================================================
 * ANSI MODE: Standard terminal output using escape sequences
 *============================================================================*/

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

#ifdef GPU_ENABLED
  /* Try to initialize GPU mode first */
  if (_gpu_init()) {
    /* GPU mode active - screen already cleared by _gpu_init */
    _initialized = true;
    return stdscr;
  }
  /* Fall through to ANSI mode if GPU init fails */
#endif

  /* Clear physical screen using ANSI codes */
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

#ifdef GPU_ENABLED
  if (_gpu_mode) {
    /* In GPU mode, just reset state - no ANSI codes needed */
    _gpu_mode = false;
    _framebuffer = NULL;
  } else
#endif
  {
    /* ANSI mode cleanup */
    /* Move cursor to bottom */
    _move_cursor(LINES - 1, 0);
    printf("\n");
    printf(ATTR_NORMAL);
    printf(CURSOR_SHOW);
    fflush(stdout);
  }

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

#ifdef GPU_ENABLED
  /* Use GPU mode refresh if active */
  if (_gpu_mode) {
    return _gpu_wrefresh(win);
  }
#endif

  /* ANSI mode refresh */
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
