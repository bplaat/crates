// Adapted from: https://mrl.cs.nyu.edu/~perlin/noise/

#pragma once

#include <stdint.h>

extern int64_t perlin_seed;

extern int32_t perlin_p[512];

extern int32_t perlin_permutation[256];

void perlin_init(int64_t seed);

double perlin_fade(double t);

double perlin_lerp(double t, double a, double b);

double perlin_grad(int32_t hash, double x, double y, double z);

double perlin_noise(double x, double y, double z);
