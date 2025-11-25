/*
 * gpu.h - GPU framebuffer interface for BogoKernel libc
 */

#ifndef _GPU_H
#define _GPU_H

/* Framebuffer info structure (must match kernel side) */
struct fb_info {
    unsigned long width;
    unsigned long height;
    unsigned long stride;
    unsigned long addr;
};

/* Get framebuffer information 
 * Returns 0 on success, -1 on failure (no framebuffer available) */
int get_fb_info(struct fb_info *info);

/* Flush framebuffer to display 
 * Returns 0 on success, -1 on failure */
int fb_flush(void);

#endif /* _GPU_H */
