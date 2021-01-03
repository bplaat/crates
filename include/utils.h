// PlaatCraft - Utils Header

#ifndef UTILS_H
#define UTILS_H

// Align Vector4 and Mattrix4 structs at 16 bytes for SSE SIMD
#define _MM_ALIGN16 __attribute__((aligned(16)))

// Function to convert radians to degrees
double degrees(double radians);

// Function to convert degrees to radians
double radians(double degrees);

char* file_read(char* path);

#endif
