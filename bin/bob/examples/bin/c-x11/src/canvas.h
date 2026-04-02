/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

#pragma once

#include <stdint.h>

#define CANVAS_COLOR(r, g, b) ((uint32_t)(((uint32_t)(r) << 16) | ((uint32_t)(g) << 8) | (uint32_t)(b)))

typedef struct canvas_t {
    int32_t width;
    int32_t height;
    uint32_t* pixels;
} canvas_t;

void canvas_init(canvas_t* canvas, int32_t width, int32_t height, uint32_t* pixels);

void canvas_fill_rect(canvas_t* canvas, int32_t x, int32_t y, int32_t w, int32_t h, uint32_t color);

void canvas_stroke_rect(canvas_t* canvas, int32_t x, int32_t y, int32_t w, int32_t h, uint32_t color);
