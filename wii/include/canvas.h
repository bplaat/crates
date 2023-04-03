#pragma once

#include <gccore.h>
#include <stdint.h>

void canvas_init(void);

void canvas_begin(uint32_t screenWidth, uint32_t screenHeight);

void canvas_end(void);

void canvas_fill_rect(float x, float y, float width, float height, uint32_t color);

void canvas_draw_image(GXTexObj *texture, float x, float y, float width, float height);

void canvas_fill_text(float x, float y, char *text, float textSize, uint32_t textColor);
