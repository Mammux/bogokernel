#include <curses.h>
#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>
#include <string.h>

/* Simple curses test application */
/* Demonstrates: boxes, borders, text attributes, positioning, and basic drawing */

void draw_header(void) {
    attron(A_BOLD);
    mvprintw(0, (COLS - 30) / 2, "BogoKernel Curses Test Demo");
    attroff(A_BOLD);
}

void draw_box_demo(int start_y, int start_x) {
    WINDOW *win = newwin(8, 30, start_y, start_x);
    if (!win) return;
    
    box(win, 0, 0);
    mvwprintw(win, 1, 2, "Box Demo");
    mvwprintw(win, 2, 2, "This is a box with");
    mvwprintw(win, 3, 2, "default borders.");
    
    attron(A_STANDOUT);
    mvwprintw(win, 5, 2, "Standout text!");
    attroff(A_STANDOUT);
    
    wrefresh(win);
    delwin(win);
}

void draw_border_demo(int start_y, int start_x) {
    WINDOW *win = newwin(8, 30, start_y, start_x);
    if (!win) return;
    
    wborder(win, '|', '|', '-', '-', '+', '+', '+', '+');
    mvwprintw(win, 1, 2, "Custom Border Demo");
    mvwprintw(win, 3, 2, "Using custom chars:");
    mvwprintw(win, 4, 2, "| - +");
    
    wrefresh(win);
    delwin(win);
}

void draw_line_demo(int start_y, int start_x) {
    mvprintw(start_y, start_x, "Line Drawing:");
    
    /* Horizontal line */
    move(start_y + 1, start_x);
    hline(ACS_HLINE, 25);
    
    /* Vertical line */
    move(start_y + 2, start_x);
    vline(ACS_VLINE, 4);
    
    mvprintw(start_y + 2, start_x + 3, "Horizontal & Vertical");
    mvprintw(start_y + 3, start_x + 3, "Line Demo");
}

void draw_attribute_demo(int start_y, int start_x) {
    mvprintw(start_y, start_x, "Attribute Demo:");
    
    move(start_y + 1, start_x);
    addstr("Normal text");
    
    move(start_y + 2, start_x);
    attron(A_BOLD);
    addstr("Bold text");
    attroff(A_BOLD);
    
    move(start_y + 3, start_x);
    attron(A_REVERSE);
    addstr("Reverse text");
    attroff(A_REVERSE);
    
    move(start_y + 4, start_x);
    attron(A_STANDOUT);
    addstr("Standout text");
    attroff(A_STANDOUT);
}

void draw_shape_demo(int start_y, int start_x) {
    /* Draw a simple filled rectangle */
    mvprintw(start_y, start_x, "Filled Rectangle:");
    
    for (int y = 0; y < 4; y++) {
        move(start_y + 2 + y, start_x + 2);
        for (int x = 0; x < 15; x++) {
            addch(ACS_BLOCK);
        }
    }
}

void draw_centered_message(int y, const char *msg) {
    int x = (COLS - strlen(msg)) / 2;
    mvprintw(y, x, "%s", msg);
}

int main(int argc, char **argv, char **envp) {
    /* Initialize curses */
    initscr();
    cbreak();
    noecho();
    
    /* Clear screen */
    clear();
    
    /* Draw header */
    draw_header();
    
    /* Draw various demos in different areas of the screen */
    draw_box_demo(2, 5);
    draw_border_demo(2, 40);
    
    draw_line_demo(11, 5);
    draw_attribute_demo(11, 40);
    
    draw_shape_demo(17, 5);
    
    /* Footer message */
    attron(A_BOLD);
    draw_centered_message(22, "Press any key to exit...");
    attroff(A_BOLD);
    
    /* Refresh to show everything */
    refresh();
    
    /* Wait for user input */
    getch();
    
    /* Clean up */
    endwin();
    
    printf("\nCurses demo completed successfully!\n");
    
    return 0;
}
