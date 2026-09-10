// PlaatCraft - Random Header

#ifndef RANDOM_H
#define RANDOM_H

#include <stdint.h>

typedef struct Random {
    int64_t seed;
} Random;

Random* random_new(int64_t seed);

double random_random(Random* random);

double random_rand(Random* random, int min, int max);

void random_free(Random* random);

#endif
