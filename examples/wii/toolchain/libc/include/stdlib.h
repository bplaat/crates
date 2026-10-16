#pragma once

#include <stddef.h>

void* malloc(size_t size);
void* calloc(size_t count, size_t size);
void* realloc(void* pointer, size_t size);
void free(void* pointer);
int abs(int value);
[[noreturn]] void exit(int status);
[[noreturn]] void abort(void);
