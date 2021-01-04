// PlaatCraft - Texture Atlas Header

#ifndef TEXTURE_ATLAS_H
#define TEXTURE_ATLAS_H

#include "glad/glad.h"

typedef struct TextureAtlas {
    char* path;
    int tileSize;
    GLuint textureArray;
} TextureAtlas;

TextureAtlas* texture_atlas_new(char* path, int tile_size);

void texture_atlas_enable(TextureAtlas* textureAtlas);

void texture_atlas_disable(TextureAtlas* textureAtlas);

void texture_atlas_free(TextureAtlas* textureAtlas);

#endif
