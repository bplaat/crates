// PlaatCraft - Font Header

#ifndef FONT_H
#define FONT_H

#include "stb_truetype/stb_truetype.h"
#include "textures/texture.h"
#include "color.h"

typedef struct Font {
    char *path;
    uint8_t *font_buffer;
    stbtt_fontinfo font_info;
} Font;

Font* font_new(char *path);

int font_measure(Font* font, char *text, int text_size);

Texture* font_render(Font* font, char *text, int text_size, Color text_color);

void font_free(Font* font);

#endif
