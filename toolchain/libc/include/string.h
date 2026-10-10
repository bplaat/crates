#pragma once

#include <stddef.h>

void* memcpy(void* restrict dst, const void* restrict src, size_t size);
void* memmove(void* dst, const void* src, size_t size);
void* memset(void* dst, int value, size_t size);
int memcmp(const void* a, const void* b, size_t size);
size_t strlen(const char* str);
size_t strnlen(const char* str, size_t max);
int strcmp(const char* a, const char* b);
char* strcpy(char* restrict dst, const char* restrict src);
char* strchr(const char* str, int c);
