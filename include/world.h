// PlaatCraft - World Header

#ifndef WORLD_H
#define WORLD_H

typedef struct World World;

#include "chunk.h"
#include "camera.h"
#include "shaders/block_shader.h"
#include "textures/texture_atlas.h"

struct World {
    int seed;

    Chunk* chunks[2048];
    int chunks_size;
};

World* world_new(int seed);

Chunk* world_get_chunk(World* world, int chunk_x, int chunk_y, int chunk_z);

void world_render(World* world, Camera* camera, BlockShader* blockShader, TextureAtlas* blocksTextureAtlas);

void world_free(World* world);

#endif
