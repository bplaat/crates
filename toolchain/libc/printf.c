// Minimal vsnprintf: supports %d %i %u %x %X %p %s %c %% with '-', '0', width and the l, ll and z
// lengths, there are no floats.

#include <stdbool.h>
#include <stdint.h>
#include <stdio.h>

typedef struct {
    char* buffer;
    size_t size;
    size_t length;
} Output;

static void put(Output* out, char c) {
    if (out->length + 1 < out->size)
        out->buffer[out->length] = c;
    out->length++;
}

static void put_padded(Output* out, const char* str, size_t length, int width, bool left, char pad) {
    if (!left) {
        for (int i = (int)length; i < width; i++)
            put(out, pad);
    }
    for (size_t i = 0; i < length; i++)
        put(out, str[i]);
    if (left) {
        for (int i = (int)length; i < width; i++)
            put(out, ' ');
    }
}

int vsnprintf(char* restrict buffer, size_t size, const char* restrict format, va_list args) {
    Output out = {buffer, size, 0};
    for (const char* c = format; *c; c++) {
        if (*c != '%') {
            put(&out, *c);
            continue;
        }
        c++;
        bool left = false;
        char pad = ' ';
        for (; *c == '-' || *c == '0'; c++) {
            if (*c == '-')
                left = true;
            else
                pad = '0';
        }
        int width = 0;
        for (; *c >= '0' && *c <= '9'; c++)
            width = width * 10 + (*c - '0');
        // long and size_t are 32 bits, long long is 64 bits
        int longs = 0;
        for (; *c == 'l' || *c == 'z'; c++)
            longs += *c == 'l';
        bool wide = longs >= 2;

        char digits[24];
        char* end = digits + sizeof(digits);
        char* start = end;
        switch (*c) {
            case 'd':
            case 'i':
            case 'u':
            case 'x':
            case 'X':
            case 'p': {
                uint64_t value;
                bool negative = false;
                if (*c == 'd' || *c == 'i') {
                    int64_t number = wide ? va_arg(args, int64_t) : va_arg(args, int32_t);
                    negative = number < 0;
                    value = negative ? -(uint64_t)number : (uint64_t)number;
                } else if (*c == 'p') {
                    value = (uintptr_t)va_arg(args, void*);
                } else {
                    value = wide ? va_arg(args, uint64_t) : va_arg(args, uint32_t);
                }
                uint32_t base = *c == 'd' || *c == 'i' || *c == 'u' ? 10 : 16;
                const char* hex = *c == 'X' ? "0123456789ABCDEF" : "0123456789abcdef";
                do {
                    *--start = hex[value % base];
                    value /= base;
                } while (value);
                if (*c == 'p') {
                    *--start = 'x';
                    *--start = '0';
                }
                if (negative) {
                    if (pad == '0') {
                        put(&out, '-');
                        width--;
                    } else {
                        *--start = '-';
                    }
                }
                put_padded(&out, start, end - start, width, left, pad);
                break;
            }
            case 's': {
                const char* str = va_arg(args, const char*);
                if (!str)
                    str = "(null)";
                size_t length = 0;
                while (str[length])
                    length++;
                put_padded(&out, str, length, width, left, ' ');
                break;
            }
            case 'c':
                *--start = (char)va_arg(args, int);
                put_padded(&out, start, 1, width, left, ' ');
                break;
            case '%':
                put(&out, '%');
                break;
            default:
                // Unknown conversion or a lone % at the end, stop here
                goto done;
        }
    }
done:
    if (size > 0)
        buffer[out.length < size ? out.length : size - 1] = '\0';
    return (int)out.length;
}

int snprintf(char* restrict buffer, size_t size, const char* restrict format, ...) {
    va_list args;
    va_start(args, format);
    int length = vsnprintf(buffer, size, format, args);
    va_end(args);
    return length;
}

int sprintf(char* restrict buffer, const char* restrict format, ...) {
    va_list args;
    va_start(args, format);
    int length = vsnprintf(buffer, SIZE_MAX, format, args);
    va_end(args);
    return length;
}
