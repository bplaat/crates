#include "log.h"
#include <stdio.h>
#include <stdlib.h>
#include <stdarg.h>

void log_debug(char* format, ...) {
    #ifdef DEBUG
        va_list args;
        va_start(args, format);
        printf("[DEBUG] ");
        vprintf(format, args);
        printf("\n");
        va_end(args);
    #endif
}

void log_info(char* format, ...) {
    va_list args;
    va_start(args, format);
    printf("[INFO] ");
    vprintf(format, args);
    printf("\n");
    va_end(args);
}

void log_warning(char* format, ...) {
    va_list args;
    va_start(args, format);
    fprintf(stderr, "[WARNING] ");
    vfprintf(stderr, format, args);
    fprintf(stderr, "\n");
    va_end(args);
}

void log_error(char* format, ...) {
    va_list args;
    va_start(args, format);
    fprintf(stderr, "[ERROR] ");
    vfprintf(stderr, format, args);
    fprintf(stderr, "\n");
    va_end(args);
    exit(EXIT_FAILURE);
}
