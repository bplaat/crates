// PlaatCraft - Utils

#include "utils.h"
#include <stdio.h>
#include <stdlib.h>
#define _USE_MATH_DEFINES
#include <math.h>

// Function to convert radians to degrees
double degrees(double radians) {
    return radians * (180 / M_PI);
}

// Function to convert degrees to radians
double radians(double degrees) {
    return degrees * (M_PI / 180);
}

// Function to read a file to string
char* file_read(char* path) {
    FILE* file = fopen(path, "r");
    fseek(file, 0, SEEK_END);
    long file_size = ftell(file);
    fseek(file, 0, SEEK_SET);
    char* file_buffer = malloc(file_size + 1);
    fread(file_buffer, 1, file_size, file);
    file_buffer[file_size] = '\0';
    fclose(file);
    return file_buffer;
}
