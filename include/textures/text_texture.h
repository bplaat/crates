// PlaatCraft - Text Texture Header

#ifndef TEXT_TEXTURE_H
#define TEXT_TEXTURE_H

#include "font.h"
#include "color.h"
#include "textures/texture.h"

typedef struct TextTexture {
    char* text;
    Texture* texture;

    Font* font;
    int text_size;
    Color text_color;
} TextTexture;

TextTexture* text_texture_new(char* text, Font* font, int text_size, Color text_color);

void text_texture_set_text(TextTexture* text_texture, char* text);

void text_texture_free(TextTexture* text_texture);

#endif
