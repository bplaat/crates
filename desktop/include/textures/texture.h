// PlaatCraft - Texture Header

#ifndef TEXTURE_H
#define TEXTURE_H

#include "glad/glad.h"
#include <stdbool.h>

typedef struct Texture {
    char* path;
    int width;
    int height;
    bool is_transparent;
    GLuint texture;
} Texture;

Texture* texture_new_from_file(char* path, bool is_transparent);

Texture* texture_new_from_bitmap(char* path, int width, int height, bool is_transparent, uint8_t* bitmap);

void texture_enable(Texture* texture);

void texture_disable(Texture* texture);

void texture_free(Texture* texture);

#endif
