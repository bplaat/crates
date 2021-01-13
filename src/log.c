// PlaatCraft - Log

#include "log.h"
#include <stdio.h>
#include <stdlib.h>
#include <stdarg.h>

mtx_t log_lock;

void log_init(void) {
    mtx_init(&log_lock, mtx_plain);
}

void log_debug(char* format, ...) {
    #ifdef DEBUG
        mtx_lock(&log_lock);

        va_list args;
        va_start(args, format);
        printf("[DEBUG] ");
        vprintf(format, args);
        printf("\n");
        va_end(args);

        mtx_unlock(&log_lock);
    #else
        (void)format;
    #endif
}

void log_info(char* format, ...) {
    mtx_lock(&log_lock);
    va_list args;
    va_start(args, format);
    printf("[INFO] ");
    vprintf(format, args);
    printf("\n");
    va_end(args);
    mtx_unlock(&log_lock);
}

void log_warning(char* format, ...) {
    mtx_lock(&log_lock);
    va_list args;
    va_start(args, format);
    fprintf(stderr, "[WARNING] ");
    vfprintf(stderr, format, args);
    fprintf(stderr, "\n");
    va_end(args);
    mtx_unlock(&log_lock);
}

void log_error(char* format, ...) {
    mtx_lock(&log_lock);
    va_list args;
    va_start(args, format);
    fprintf(stderr, "[ERROR] ");
    vfprintf(stderr, format, args);
    fprintf(stderr, "\n");
    va_end(args);
    mtx_unlock(&log_lock);
    exit(EXIT_FAILURE);
}
