#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>

/* Simple test without curses to verify basic I/O */

int main(int argc, char **argv, char **envp) {
    printf("Simple test starting...\n");
    printf("Press any key to continue...\n");
    
    char c;
    if (read(0, &c, 1) == 1) {
        printf("You pressed: %c (0x%02x)\n", c, (unsigned char)c);
    }
    
    printf("Test completed!\n");
    return 0;
}
