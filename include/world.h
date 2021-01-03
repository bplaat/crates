// PlaatCraft - World Header

#ifndef WORLD_H
#define WORLD_H

typedef struct World World;

#include "chunk.h"
#include "camera.h"
#include "shader.h"
#include "texture_atlas.h"

struct World {
    int seed;

    Chunk* chunks[512];
    int chunks_size;
};

World* world_new(int seed);

Chunk* world_get_chunk(World* world, int chunk_x, int chunk_y, int chunk_z);

void world_render(World* world, Camera* camera, Shader* blockShader, TextureAtlas* blocksTextureAtlas);

void world_free(World* world);

#endif
