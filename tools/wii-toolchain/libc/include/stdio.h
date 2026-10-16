#pragma once

#include <stdarg.h>
#include <stddef.h>

int vsnprintf(char* restrict buffer, size_t size, const char* restrict format, va_list args);
int snprintf(char* restrict buffer, size_t size, const char* restrict format, ...);
int sprintf(char* restrict buffer, const char* restrict format, ...);

typedef struct FILE FILE;
#define stdout ((FILE*)0)
#define stderr ((FILE*)0)

// There is no console, so printing does nothing, the arguments are still evaluated
static inline int printf(const char* restrict format, ...) {
    (void)format;
    return 0;
}
static inline int fprintf(FILE* restrict file, const char* restrict format, ...) {
    (void)file, (void)format;
    return 0;
}

// There is no filesystem, so opening a file always fails
#define SEEK_SET 0
#define SEEK_CUR 1
#define SEEK_END 2
static inline FILE* fopen(const char* path, const char* mode) {
    (void)path, (void)mode;
    return NULL;
}
static inline size_t fread(void* buffer, size_t size, size_t count, FILE* file) {
    (void)buffer, (void)size, (void)count, (void)file;
    return 0;
}
static inline int fseek(FILE* file, long offset, int whence) {
    (void)file, (void)offset, (void)whence;
    return -1;
}
static inline int fclose(FILE* file) {
    (void)file;
    return 0;
}
