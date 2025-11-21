#include <stddef.h>

// Defined in syscalls.c
long write(int fd, const void* buf, size_t count);
void exit(int code);

int strlen(const char* s) {
    int len = 0;
    while (s[len]) len++;
    return len;
}

int main() {
    const char* msg = "Hello from C World!\n";
    write(1, msg, strlen(msg));
    return 0;
}
