#include <stdlib.h>

int abs(int value) {
    return value < 0 ? -value : value;
}

void abort(void) {
    exit(1);
}
