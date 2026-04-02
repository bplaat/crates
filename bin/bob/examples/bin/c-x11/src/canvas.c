/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

#include "canvas.h"

void canvas_init(canvas_t* canvas, int32_t width, int32_t height, uint32_t* pixels, float scale) {
    canvas->width = width;
    canvas->height = height;
    canvas->scale = scale > 0.0f ? scale : 1.0f;
    canvas->phys_width = (int32_t)((float)width * canvas->scale);
    canvas->phys_height = (int32_t)((float)height * canvas->scale);
    canvas->pixels = pixels;
}

void canvas_fill_rect(canvas_t* canvas, float x, float y, float w, float h, uint32_t color) {
    // All math in float; only convert to int at the pixel boundary
    float px = x * canvas->scale;
    float py = y * canvas->scale;
    float px2 = px + w * canvas->scale;
    float py2 = py + h * canvas->scale;

    // Clip to physical canvas bounds
    if (px < 0.0f) px = 0.0f;
    if (py < 0.0f) py = 0.0f;
    if (px2 > (float)canvas->phys_width) px2 = (float)canvas->phys_width;
    if (py2 > (float)canvas->phys_height) py2 = (float)canvas->phys_height;
    if (px >= px2 || py >= py2)
        return;

    int32_t x1 = (int32_t)px;
    int32_t y1 = (int32_t)py;
    int32_t x2 = (int32_t)px2;
    int32_t y2 = (int32_t)py2;
    int32_t stride = canvas->phys_width;
    int32_t count = x2 - x1;
    for (int32_t row = y1; row < y2; row++) {
        uint32_t* dst = canvas->pixels + row * stride + x1;
        for (int32_t col = 0; col < count; col++) {
            dst[col] = color;
        }
    }
}

void canvas_stroke_rect(canvas_t* canvas, float x, float y, float w, float h, float line_width, uint32_t color) {
    if (w <= 0.0f || h <= 0.0f || line_width <= 0.0f)
        return;
    float lw = line_width;
    // Top and bottom edges
    canvas_fill_rect(canvas, x, y, w, lw, color);
    canvas_fill_rect(canvas, x, y + h - lw, w, lw, color);
    // Left and right edges (excluding corners already drawn)
    if (h > lw * 2.0f) {
        canvas_fill_rect(canvas, x, y + lw, lw, h - lw * 2.0f, color);
        canvas_fill_rect(canvas, x + w - lw, y + lw, lw, h - lw * 2.0f, color);
    }
}
