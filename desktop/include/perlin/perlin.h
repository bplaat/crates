// PlaatCraft - Perlin Noise Header
// Adapted from: https://mrl.cs.nyu.edu/~perlin/noise/

#ifndef PERLIN_H
#define PERLIN_H

#include <stdint.h>

extern int64_t perlin_seed;

extern int perlin_p[512];

extern int perlin_permutation[256];

void perlin_init(int64_t seed);

double perlin_fade(double t);

double perlin_lerp(double t, double a, double b);

double perlin_grad(int hash, double x, double y, double z);

double perlin_noise(double x, double y, double z);

#endif
