// PlaatCraft - Utils

#include "utils.h"
#include <stdio.h>
#include <stdlib.h>
#include <stdint.h>
#include <string.h>
#include "log.h"

// Function to convert radians to degrees
double degrees(double radians) {
    return radians * (180 / M_PI);
}

// Function to convert degrees to radians
double radians(double degrees) {
    return degrees * (M_PI / 180);
}

// Function to read a file to string
uint8_t* file_read(char* path) {
    FILE* file = fopen(path, "r");
    if (file == NULL) {
        log_error("Can't load file %s", path);
    }
    fseek(file, 0, SEEK_END);
    size_t file_size = ftell(file);
    fseek(file, 0, SEEK_SET);
    uint8_t* file_buffer = malloc(file_size + 1);
    fread(file_buffer, 1, file_size, file);
    file_buffer[file_size] = 0;
    fclose(file);
    return file_buffer;
}

// Function to copy / clone a string
char *string_copy(char *string) {
    char *new_string = malloc(strlen(string) + 1);
    strcpy(new_string, string);
    return new_string;
}
