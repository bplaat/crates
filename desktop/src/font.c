// PlaatCraft - Font

#include "font.h"
#include <stdlib.h>
#include <string.h>
#include "utils.h"
#include "log.h"

Font* font_new(char* path) {
    Font* font = malloc(sizeof(Font));
    font->path = path;
    font->font_buffer = file_read(path);

    if (stbtt_InitFont(&font->font_info, font->font_buffer, stbtt_GetFontOffsetForIndex(font->font_buffer, 0)) == 0) {
        log_error("Can't load font %s", path);
    }

    return font;
}

int font_measure(Font* font, char* text, int text_size) {
    float scale = stbtt_ScaleForPixelHeight(&font->font_info, text_size);
    int x_padding = text_size / 4;
    int width = x_padding * 2;
    for (size_t i = 0; i < strlen(text); i++) {
        int advance, lsb;
        stbtt_GetCodepointHMetrics(&font->font_info, text[i], &advance, &lsb);

        width += advance * scale;

        if (text[i + 1]) {
            width += stbtt_GetCodepointKernAdvance(&font->font_info, text[i], text[i + 1]) * scale;
        }
    }
    return width;
}

// BUGGY
Texture* font_render(Font* font, char *text, int text_size, Color text_color) {
    float scale = stbtt_ScaleForPixelHeight(&font->font_info, text_size);
    int x_padding = text_size / 4;

    int width = font_measure(font, text, text_size);
    int height = text_size;

    uint8_t* bitmap = malloc(height * width);
    for (int i = 0; i < height * width; i++) {
        bitmap[i] = 0;
    }

    int x = x_padding;

    int ascent, descent, line_gap;
    stbtt_GetFontVMetrics(&font->font_info, &ascent, &descent, &line_gap);

    ascent = roundf(ascent * scale);
    descent = roundf(descent * scale);

    for (size_t i = 0; i < strlen(text); i++) {
        int advance, lsb;
        stbtt_GetCodepointHMetrics(&font->font_info, text[i], &advance, &lsb);

        int c_x1, c_y1, c_x2, c_y2;
        stbtt_GetCodepointBitmapBox(&font->font_info, text[i], scale, scale, &c_x1, &c_y1, &c_x2, &c_y2);
        int y = ascent + c_y1;

        int character_width, character_height;
        uint8_t* character_bitmap = stbtt_GetCodepointBitmap(&font->font_info, 0, scale, text[i], &character_width, &character_height, 0, 0);
        for (int cy = 0; cy < character_height; cy++) {
            for (int cx = 0; cx < character_width; cx++) {
                int pos = (y + cy) * width + (x + cx);
                bitmap[pos] = MAX(bitmap[pos], character_bitmap[cy * character_width + cx]);
            }
        }
        stbtt_FreeBitmap(character_bitmap, NULL);

        x += advance * scale;

        if (text[i + 1]) {
            x += stbtt_GetCodepointKernAdvance(&font->font_info, text[i], text[i + 1]) * scale;
        }
    }

    uint8_t* colored_bitmap = malloc(height * width * 4);
    for (int y = 0; y < height; y++) {
        for (int x = 0; x < width; x++) {
            int pos = (y * width + x) * 4;
            colored_bitmap[pos] = text_color.red;
            colored_bitmap[pos + 1] = text_color.green;
            colored_bitmap[pos + 2] = text_color.blue;
            colored_bitmap[pos + 3] = bitmap[y * width + x];
        }
    }

    free(bitmap);

    Texture* texture = texture_new_from_bitmap("Text", width, height, true, colored_bitmap);

    free(colored_bitmap);

    return texture;
}

void font_free(Font* font) {
    free(font->font_buffer);
    free(font);
}
