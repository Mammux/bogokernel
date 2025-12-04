/*
 * font.h - Shared 8x16 VGA-style bitmap font
 * 
 * This font data is shared between kernel and libc for consistent rendering.
 * 
 * Each character is represented by 16 bytes, one per row
 * Bit 7 (MSB) is the leftmost pixel, bit 0 (LSB) is the rightmost pixel
 */

#ifndef _FONT_H
#define _FONT_H

#define FONT_WIDTH 8
#define FONT_HEIGHT 16

/* 8x16 VGA-style font data for ASCII characters 32-126 (95 characters) */
extern const unsigned char FONT_8X16[95][16];

/* Get font bitmap for a character (returns pointer to 16-byte array or NULL) */
const unsigned char* get_char_bitmap(unsigned char c);

#endif /* _FONT_H */
