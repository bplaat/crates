// PlaatCraft - Utils Header

#ifndef UTILS_H
#define UTILS_H

#include <math.h>
#include "glad/glad.h"
#include "log.h"

// Define PI when not defined
#ifndef M_PI
    #define M_PI 3.14159265358979323846
#endif

// Define min max macro's
#define MAX(x, y) (((x) > (y)) ? (x) : (y))
#define MIN(x, y) (((x) < (y)) ? (x) : (y))

// Align Vector4 and Mattrix4 structs at 16 bytes for SSE SIMD
#ifndef NO_SSIMD
    #define _MM_ALIGN16 __attribute__((aligned(16)))
#endif

// Function to convert radians to degrees
double degrees(double radians);

// Function to convert degrees to radians
double radians(double degrees);

// Function to read a file to string
uint8_t* file_read(char* path);

// Function to copy / clone a string
char *string_copy(char *string);

#endif
