#pragma once
#include <stdint.h>

typedef int64_t time_t;

struct timespec {
    time_t tv_sec;
    long tv_nsec;
};

int nanosleep(const struct timespec* request, struct timespec* remaining);
