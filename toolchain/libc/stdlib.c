#include <stdlib.h>

#include "libogc.h"

int abs(int value) {
    return value < 0 ? -value : value;
}

void exit(int status) {
    __syscall_exit(status);
    for (;;)
        ;
}

// Crashes with a trap, so the exception handler shows its red screen
void abort(void) {
    __builtin_trap();
}
