// PlaatCraft - Texture Header

#ifndef TEXTURE_H
#define TEXTURE_H

#include "glad.h"
#include <stdbool.h>

typedef struct Texture {
    char* path;
    int width;
    int height;
    bool isTransparent;
    GLuint texture;
} Texture;

Texture* texture_new(char* path, bool isTransparent);

void texture_enable(Texture* texture);

void texture_disable(Texture* texture);

void texture_free(Texture* texture);

#endif
