#include "random.h"
#include <math.h>

uint64_t random_seed;

double random_random(void) {
    double x = sin(random_seed++) * 10000;
    return x - floor(x);
}

int32_t random_rand(int32_t min, int32_t max) {
    return (int32_t)(random_random() * (max - min + 1)) + min;
}
