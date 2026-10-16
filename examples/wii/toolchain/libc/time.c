#include <time.h>
#include <unistd.h>

#include "libogc.h"

int usleep(unsigned int microseconds) {
    struct timespec request = {microseconds / 1000000, (long)(microseconds % 1000000) * 1000};
    return __syscall_nanosleep(&request, NULL);
}
