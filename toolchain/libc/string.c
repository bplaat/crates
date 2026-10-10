#include <stdint.h>
#include <string.h>

// Words are copied and set 4 bytes at a time, which is 4 times fewer loads and stores than bytes.
// The 8 byte floating point registers aren't used, because interrupt code may not touch them.
typedef uint32_t __attribute__((may_alias)) word;

static void copy_forward(uint8_t* d, const uint8_t* s, size_t size) {
    if (size >= 16 && (((uintptr_t)d ^ (uintptr_t)s) & 3) == 0) {
        for (; (uintptr_t)d & 3; size--)
            *d++ = *s++;
        word* dw = (word*)d;
        const word* sw = (const word*)s;
        for (; size >= 16; size -= 16, dw += 4, sw += 4) {
            dw[0] = sw[0];
            dw[1] = sw[1];
            dw[2] = sw[2];
            dw[3] = sw[3];
        }
        for (; size >= 4; size -= 4)
            *dw++ = *sw++;
        d = (uint8_t*)dw;
        s = (const uint8_t*)sw;
    }
    while (size--)
        *d++ = *s++;
}

static void copy_backward(uint8_t* d, const uint8_t* s, size_t size) {
    d += size;
    s += size;
    if (size >= 16 && (((uintptr_t)d ^ (uintptr_t)s) & 3) == 0) {
        for (; (uintptr_t)d & 3; size--)
            *--d = *--s;
        word* dw = (word*)d;
        const word* sw = (const word*)s;
        for (; size >= 4; size -= 4)
            *--dw = *--sw;
        d = (uint8_t*)dw;
        s = (const uint8_t*)sw;
    }
    while (size--)
        *--d = *--s;
}

void* memcpy(void* restrict dst, const void* restrict src, size_t size) {
    copy_forward(dst, src, size);
    return dst;
}

void* memmove(void* dst, const void* src, size_t size) {
    if ((uintptr_t)dst <= (uintptr_t)src || (uintptr_t)dst >= (uintptr_t)src + size)
        copy_forward(dst, src, size);
    else
        copy_backward(dst, src, size);
    return dst;
}

// Zeroes the 32 byte cache line at line, without reading it from memory first
static inline void zero_cache_line(word* line) {
    __asm__ volatile("dcbz 0,%0" : : "b"(line) : "memory");
}

void* memset(void* dst, int value, size_t size) {
    uint8_t* d = dst;
    uint8_t byte = (uint8_t)value;
    if (size >= 16) {
        for (; (uintptr_t)d & 3; size--)
            *d++ = byte;
        word fill = byte * 0x01010101u;
        word* dw = (word*)d;
        // dcbz only works on cached memory: 0x8... (MEM1) and 0x9... (MEM2)
        if (fill == 0 && size >= 64 && ((uintptr_t)dw >> 28 == 0x8 || (uintptr_t)dw >> 28 == 0x9)) {
            for (; (uintptr_t)dw & 31; size -= 4)
                *dw++ = 0;
            for (; size >= 32; size -= 32, dw += 8)
                zero_cache_line(dw);
        }
        for (; size >= 16; size -= 16, dw += 4)
            dw[0] = dw[1] = dw[2] = dw[3] = fill;
        for (; size >= 4; size -= 4)
            *dw++ = fill;
        d = (uint8_t*)dw;
    }
    while (size--)
        *d++ = byte;
    return dst;
}

int memcmp(const void* a, const void* b, size_t size) {
    const unsigned char *x = a, *y = b;
    for (size_t i = 0; i < size; i++) {
        if (x[i] != y[i])
            return x[i] - y[i];
    }
    return 0;
}

size_t strlen(const char* str) {
    size_t length = 0;
    while (str[length])
        length++;
    return length;
}

size_t strnlen(const char* str, size_t max) {
    size_t length = 0;
    while (length < max && str[length])
        length++;
    return length;
}

int strcmp(const char* a, const char* b) {
    while (*a && *a == *b)
        a++, b++;
    return (unsigned char)*a - (unsigned char)*b;
}

int strncmp(const char* a, const char* b, size_t max) {
    for (size_t i = 0; i < max; i++) {
        if (a[i] != b[i] || !a[i])
            return (unsigned char)a[i] - (unsigned char)b[i];
    }
    return 0;
}

char* strcpy(char* restrict dst, const char* restrict src) {
    char* d = dst;
    while ((*d++ = *src++))
        ;
    return dst;
}

char* strcat(char* restrict dst, const char* restrict src) {
    strcpy(dst + strlen(dst), src);
    return dst;
}

char* strchr(const char* str, int c) {
    for (;; str++) {
        if (*str == (char)c)
            return (char*)str;
        if (!*str)
            return NULL;
    }
}
