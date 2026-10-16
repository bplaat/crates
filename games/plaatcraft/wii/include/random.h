#pragma once

#include <stdint.h>

extern uint64_t random_seed;

double random_random(void);

int32_t random_rand(int32_t min, int32_t max);
