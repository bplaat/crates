#include "log.h"
#include <stdio.h>
#include <stdlib.h>
#include <stdarg.h>

pthread_mutex_t log_lock;

void log_init() {
    pthread_mutex_init(&log_lock, NULL);
}

void log_debug(char* format, ...) {
    #ifdef DEBUG
        pthread_mutex_lock(&log_lock);

        va_list args;
        va_start(args, format);
        printf("[DEBUG] ");
        vprintf(format, args);
        printf("\n");
        va_end(args);

        pthread_mutex_unlock(&log_lock);
    #endif
}

void log_info(char* format, ...) {
    pthread_mutex_lock(&log_lock);
    va_list args;
    va_start(args, format);
    printf("[INFO] ");
    vprintf(format, args);
    printf("\n");
    va_end(args);
    pthread_mutex_unlock(&log_lock);
}

void log_warning(char* format, ...) {
    pthread_mutex_lock(&log_lock);
    va_list args;
    va_start(args, format);
    fprintf(stderr, "[WARNING] ");
    vfprintf(stderr, format, args);
    fprintf(stderr, "\n");
    va_end(args);
    pthread_mutex_unlock(&log_lock);
}

void log_error(char* format, ...) {
    pthread_mutex_lock(&log_lock);
    va_list args;
    va_start(args, format);
    fprintf(stderr, "[ERROR] ");
    vfprintf(stderr, format, args);
    fprintf(stderr, "\n");
    va_end(args);
    pthread_mutex_unlock(&log_lock);
    exit(EXIT_FAILURE);
}
