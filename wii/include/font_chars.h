#pragma once

#include <stdint.h>

typedef struct FontChar {
    uint8_t x;
    uint8_t y;
    uint8_t width;
    uint8_t height;
    int8_t originX;
    int8_t originY;
} FontChar;

extern FontChar font_chars[];
