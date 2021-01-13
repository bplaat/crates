// PlaatCraft - Text Texture

#include "textures/text_texture.h"
#include <stdlib.h>

TextTexture* text_texture_new(char* text, Font* font, int text_size, Color text_color) {
    TextTexture* text_texture = malloc(sizeof(TextTexture));
    text_texture->texture = NULL;

    text_texture->font = font;
    text_texture->text_size = text_size;
    text_texture->text_color = text_color;

    text_texture_set_text(text_texture, text);

    return text_texture;
}

void text_texture_set_text(TextTexture* text_texture, char* text) {
    text_texture->text = text;

    if (text_texture->texture != NULL) {
        texture_free(text_texture->texture);
    }

    text_texture->texture = font_render(text_texture->font, text_texture->text, text_texture->text_size, text_texture->text_color);
}

void text_texture_free(TextTexture* text_texture) {
    if (text_texture->texture != NULL) {
        texture_free(text_texture->texture);
    }

    free(text_texture);
}
