# Curses Library Implementation Notes

This document describes the curses/ncurses implementation in BogoKernel for developers and AI agents working on the library.

## Location

- **Header**: `libc/include/curses.h`
- **Implementation**: `libc/src/curses.c`
- **Build**: Part of `libc/libc.a`

## Architecture

The curses implementation uses a **screen buffering** approach with dirty flag tracking for efficient terminal updates.

### Key Data Structures

```c
// Global screen buffers (in curses.c)
static char screen_buffer[LINES][COLS];      // Character buffer (24x80)
static chtype attr_buffer[LINES][COLS];      // Attribute buffer
static bool dirty[LINES][COLS];              // Dirty flags for updates

// Window structure
typedef struct _win_st {
    int _cury, _curx;        // Current cursor position (window-relative)
    int _maxy, _maxx;        // Window dimensions
    int _begy, _begx;        // Window position on screen
    chtype _attrs;           // Current attributes
    bool _clear;             // Clear flag
    // ... other fields
} WINDOW;

// Global windows
WINDOW *stdscr;  // Standard screen (covers full terminal)
WINDOW *curscr;  // Current screen state
```

### Coordinate Systems

1. **Window Coordinates**: Relative to window (0,0) is top-left of window
2. **Screen Coordinates**: Absolute screen position
   - Conversion: `screen_y = win->_begy + win_y`
   - Conversion: `screen_x = win->_begx + win_x`

## Critical Functions

### wrefresh(WINDOW *win)

**Purpose**: Updates the physical terminal with dirty cells from the screen buffer.

**Important**: As of the bug fix, `wrefresh()` now only processes and clears dirty flags within the window's bounds, not the entire screen.

```c
int wrefresh(WINDOW *win) {
    // Calculate window bounds in screen coordinates
    int win_top = win->_begy;
    int win_bottom = win->_begy + win->_maxy;
    int win_left = win->_begx;
    int win_right = win->_begx + win->_maxx;
    
    // Clamp to screen bounds
    if (win_top < 0) win_top = 0;
    if (win_bottom > LINES) win_bottom = LINES;
    // ... etc
    
    // Update only dirty cells within window bounds
    for (int y = win_top; y < win_bottom; y++) {
        for (int x = win_left; x < win_right; x++) {
            if (dirty[y][x]) {
                // Output character and clear dirty flag
                dirty[y][x] = false;
            }
        }
    }
}
```

**Key Behavior**:
- Only clears dirty flags for cells within the window being refreshed
- For `stdscr`, this covers the entire screen (0-23, 0-79)
- For subwindows, only affects their area
- Prevents interference between window refreshes

### touchwin(WINDOW *win)

**Purpose**: Marks all cells in a window as dirty, forcing them to be redrawn on next refresh.

```c
int touchwin(WINDOW *win) {
    // Mark all cells in the window as dirty
    for (int y = win->_begy; y < win->_begy + win->_maxy && y < LINES; y++) {
        for (int x = win->_begx; x < win->_begx + win->_maxx && x < COLS; x++) {
            dirty[y][x] = true;
        }
    }
    return 0;
}
```

**Usage**: Call after operations that should force a full window redraw.

### clearok(WINDOW *win, bool bf)

**Purpose**: Sets the clear flag and optionally marks all screen cells as dirty.

```c
int clearok(WINDOW *win, bool bf) {
    win->_clear = bf;
    if (bf) {
        // Mark ALL cells as dirty (entire screen)
        memset(dirty, 1, sizeof(dirty));
    }
    return 0;
}
```

**Important**: When `bf` is TRUE, marks the ENTIRE screen as dirty, not just the window.

### waddch(WINDOW *win, chtype ch)

**Purpose**: Adds a character to the window buffer.

**Key Behavior**:
- Converts window coordinates to screen coordinates
- Stores character in `screen_buffer[y][x]`
- Stores attributes in `attr_buffer[y][x]`
- Sets `dirty[y][x] = true`
- Advances cursor position

## Common Patterns

### Pattern 1: Full Screen Redraw

```c
clearok(curscr, TRUE);  // Mark all cells dirty
touchwin(stdscr);       // Redundant but safe
refresh();              // Redraw everything
```

### Pattern 2: Overlay Window

```c
WINDOW *overlay = newwin(height, width, y, x);
waddstr(overlay, "Content");
wrefresh(overlay);      // Updates overlay area only
delwin(overlay);
touchwin(stdscr);       // Mark background for redraw
refresh();              // Redraw background
```

### Pattern 3: Inventory/Menu Display

This is the pattern used by bigrogue's inventory system:

```c
// Create temporary overlay window
WINDOW *tw = newwin(lines, cols, y, x);

// Fill with content
mvwprintw(tw, 0, 0, "Inventory:");
// ... add items ...

// Display and wait
touchwin(tw);
wrefresh(tw);          // Show overlay (clears dirty in overlay area)
wait_for_key(' ');

// Clean up
werase(tw);            // Optional: clear the window
wrefresh(tw);          // Optional: update terminal
delwin(tw);            // Delete window

// Restore background
touchwin(stdscr);      // Mark stdscr dirty
refresh();             // Redraw entire screen
```

## Bug That Was Fixed

**Problem**: After closing the inventory in bigrogue, only map spaces adjacent to the player were redrawn.

**Root Cause**: `wrefresh()` was iterating over ALL screen cells (0 to LINES, 0 to COLS) and clearing all dirty flags, regardless of which window was being refreshed. This meant:

1. Inventory overlay window created and refreshed → cleared all dirty flags
2. `touchwin(stdscr)` called → marked stdscr cells as dirty
3. But if another `wrefresh()` call happened (on the overlay during cleanup), it would clear stdscr's dirty flags
4. Final `refresh()` found no dirty cells to redraw

**Solution**: Modified `wrefresh()` to only iterate over and clear dirty flags within the window's bounds. Now:
1. `wrefresh(overlay)` only clears dirty flags in overlay area
2. `touchwin(stdscr)` marks background dirty
3. `refresh()` redraws the full screen

## Testing Curses Changes

### Test Application

Use `curses_test/curses_test.c` to test basic curses functionality:

```bash
cd curses_test
bash build.sh
cp curses_test.elf ../kernel/
cd ..
cargo build -p kernel
cargo run -p kernel
# In shell: curses_test
```

### Manual Test: Bigrogue Inventory

```bash
cargo run -p kernel
# In shell: bigrogue
# In game: press 'i' for inventory
# Press space to close
# Verify: entire map is redrawn, not just adjacent cells
```

## Performance Considerations

- **Dirty Flag Optimization**: Only cells marked dirty are updated on refresh
- **Attribute Caching**: Attribute changes only sent when needed
- **Cursor Positioning**: ANSI escape sequences used for efficient cursor movement
- **Buffer Flusing**: `fflush()` called after refresh to ensure output is sent

## Known Limitations

1. **No Hardware Scrolling**: `idlok()` is a no-op
2. **No Input Buffering**: `flushinp()` is a no-op  
3. **No Keypad Translation**: Arrow keys not translated to KEY_* codes
4. **No Color Support**: Only basic attributes (bold, reverse, standout)
5. **No Mouse Support**: Not implemented
6. **Single Screen Buffer**: No multiple screen support

## ANSI Escape Sequences Used

```c
#define CLEAR_SCREEN    "\033[2J\033[H"    // Clear and home
#define CURSOR_HOME     "\033[H"            // Home cursor
#define CURSOR_HIDE     "\033[?25l"         // Hide cursor
#define CURSOR_SHOW     "\033[?25h"         // Show cursor
#define ATTR_NORMAL     "\033[0m"           // Reset attributes
#define ATTR_BOLD       "\033[1m"           // Bold text
#define ATTR_REVERSE    "\033[7m"           // Reverse video
#define CLEAR_EOL       "\033[K"            // Clear to end of line

// Cursor positioning
printf("\033[%d;%dH", row + 1, col + 1);   // Move cursor
```

## Adding New Curses Features

1. Add function declaration to `libc/include/curses.h`
2. Implement function in `libc/src/curses.c`
3. Follow existing patterns for:
   - NULL pointer checks
   - Bounds checking
   - Dirty flag management
   - Window coordinate conversion
4. Rebuild libc: `cd libc && bash build.sh`
5. Rebuild dependent applications
6. Rebuild kernel
7. Test with `curses_test` or actual applications

## Debugging Tips

### Common Issues

1. **Characters not appearing**: Check if cell is marked dirty and `wrefresh()` is called
2. **Wrong coordinates**: Verify window vs. screen coordinate conversion
3. **Attributes not working**: Check attribute buffer and `_set_attrs()` logic
4. **Window overlap issues**: Use `touchwin()` to mark affected windows dirty
5. **Memory corruption**: Check bounds on all screen_buffer/attr_buffer/dirty accesses

### Debug Additions

Add temporary debug output:

```c
// In curses.c, add debug function:
void debug_dirty_flags() {
    for (int y = 0; y < LINES; y++) {
        for (int x = 0; x < COLS; x++) {
            if (dirty[y][x]) {
                printf("D");
            } else {
                printf(".");
            }
        }
        printf("\n");
    }
}
```

Call before/after key operations to visualize dirty flag state.

## Related Documentation

- See `rogue/pack.c` for inventory system that uses curses
- See `rogue/things.c` for `add_line()` function that displays inventory
- See main `README.md` for overall project documentation
- See `BUILD_GUIDE.md` for build instructions
