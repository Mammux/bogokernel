#include <stdio.h>
#include <stdlib.h>

int main(int argc, char **argv, char **envp) {
    printf("Hello from C with libc!\n");
    printf("argc = %d\n", argc);
    
    for (int i = 0; i < argc; i++) {
        printf("argv[%d] = %s\n", i, argv[i]);
    }
    
    return 0;
}
