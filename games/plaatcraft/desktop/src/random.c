// PlaatCraft - Random

#include "random.h"
#include <stdlib.h>
#include <math.h>

Random* random_new(int64_t seed) {
    Random* random = malloc(sizeof(Random));
    random->seed = seed;
    return random;
}

double random_random(Random* random) {
    double x = sin(random->seed++) * 10000;
    return x - floor(x);
}

double random_rand(Random* random, int min, int max) {
    return (int)(random_random(random) * (max - min + 1)) + min;
}

void random_free(Random* random) {
    free(random);
}
