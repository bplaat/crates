// PlaatCraft - Utils Header

#ifndef UTILS_H
#define UTILS_H

#include "glad/glad.h"

// Align Vector4 and Mattrix4 structs at 16 bytes for SSE1 SIMD
#ifndef NO_SSIMD
    #define _MM_ALIGN16 __attribute__((aligned(16)))
#endif

// Function to convert radians to degrees
double degrees(double radians);

// Function to convert degrees to radians
double radians(double degrees);

// Function to read a file to string
char* file_read(char* path);

#endif
