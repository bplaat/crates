/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

#include "canvas.h"

#include <string.h>
#include <wchar.h>

void canvas_init(canvas_t* canvas, int32_t width, int32_t height, uint32_t* pixels) {
    canvas->width = width;
    canvas->height = height;
    canvas->pixels = pixels;
}

void canvas_fill_rect(canvas_t* canvas, int32_t x, int32_t y, int32_t w, int32_t h, uint32_t color) {
    // Clip to canvas bounds
    int32_t x1 = x < 0 ? 0 : x;
    int32_t y1 = y < 0 ? 0 : y;
    int32_t x2 = x + w > canvas->width ? canvas->width : x + w;
    int32_t y2 = y + h > canvas->height ? canvas->height : y + h;
    if (x1 >= x2 || y1 >= y2)
        return;

    int32_t count = x2 - x1;
    for (int32_t row = y1; row < y2; row++) {
        wmemset((wchar_t*)(canvas->pixels + row * canvas->width + x1), (wchar_t)color, (size_t)count);
    }
}

void canvas_stroke_rect(canvas_t* canvas, int32_t x, int32_t y, int32_t w, int32_t h, uint32_t color) {
    if (w <= 0 || h <= 0)
        return;
    // Top and bottom edges
    canvas_fill_rect(canvas, x, y, w, 1, color);
    canvas_fill_rect(canvas, x, y + h - 1, w, 1, color);
    // Left and right edges (excluding corners already drawn)
    if (h > 2) {
        canvas_fill_rect(canvas, x, y + 1, 1, h - 2, color);
        canvas_fill_rect(canvas, x + w - 1, y + 1, 1, h - 2, color);
    }
}
