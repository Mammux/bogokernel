# Curses Test Application

This directory contains a test application demonstrating the BogoKernel curses library.

## Features Demonstrated

The `curses_test.c` application demonstrates:

1. **Box Drawing** - Using `box()` to draw borders around windows
2. **Custom Borders** - Using `wborder()` with custom characters
3. **Line Drawing** - Horizontal and vertical lines using `hline()` and `vline()`
4. **Text Attributes** - Bold, reverse, and standout text modes
5. **Filled Shapes** - Drawing filled rectangles using ACS_BLOCK
6. **Window Management** - Creating and managing multiple windows with `newwin()`
7. **Cursor Positioning** - Using `mvprintw()` for precise text placement

## Curses Implementation Updates

The following enhancements were made to the curses library:

### curses.h
- Added ACS (Alternative Character Set) defines for line drawing:
  - `ACS_ULCORNER`, `ACS_URCORNER`, `ACS_LLCORNER`, `ACS_LRCORNER` - corners
  - `ACS_HLINE`, `ACS_VLINE` - horizontal and vertical lines
  - `ACS_BLOCK`, `ACS_BULLET`, `ACS_CKBOARD` - special characters
- Added box drawing function declarations:
  - `box()`, `border()`, `wborder()` - border drawing
  - `hline()`, `vline()`, `whline()`, `wvline()` - line drawing
  - `mvhline()`, `mvvline()`, `mvwhline()`, `mvwvline()` - positioned line drawing

### curses.c
- Implemented all box and line drawing functions
- Functions support both default and custom characters
- Proper window coordinate handling and bounds checking

### stdio.h/stdio.c
- Added `stdout` FILE* variable for compatibility
- Implemented `vsnprintf()` for formatted string output to buffers
- Implemented `fflush()` (no-op since output is unbuffered)
- Added `NULL` define for compatibility

## Building

```bash
cd curses_test
./build.sh
```

This creates `curses_test.elf` which can be copied to the kernel directory.

## Integration

The application has been:
1. Built and copied to `kernel/curses_test.elf`
2. Added to the kernel RAMFS in `kernel/src/fs.rs`
3. Integrated into the shell in `userapp/src/bin/shell.rs`

## Running

From the BogoShell:
```
> curses_test
```

The application will:
1. Initialize curses mode
2. Clear the screen
3. Draw various demonstration windows and shapes
4. Wait for a keypress
5. Clean up and return to the shell

## Layout

The curses test displays:
- Header: "BogoKernel Curses Test Demo" (centered, bold)
- Top left: Box demo with standout text
- Top right: Custom border demo
- Middle left: Line drawing demonstration
- Middle right: Attribute demonstration (normal, bold, reverse, standout)
- Bottom left: Filled rectangle shape
- Bottom: "Press any key to exit..." message

## Notes

- The curses library uses ANSI escape sequences for terminal control
- Colors are not yet implemented (only attributes like bold, reverse)
- The implementation is optimized for the BogoKernel UART console
- Screen buffer tracks changes to minimize output (dirty cell tracking)
