// PlaatCraft - Block Header

#ifndef BLOCK_H
#define BLOCK_H

#include "glad.h"

typedef enum BlockTexture {
    BLOCK_TEXTURE_GRASS_TOP,
    BLOCK_TEXTURE_GRASS_SIDE,
    BLOCK_TEXTURE_DIRT,
    BLOCK_TEXTURE_WATER,
    BLOCK_TEXTURE_SAND,
    BLOCK_TEXTURE_STONE,
    BLOCK_TEXTURE_COAL,
    BLOCK_TEXTURE_GOLD,
    BLOCK_TEXTURE_TRUNK_TOP,
    BLOCK_TEXTURE_TRUNK_SIDE,
    BLOCK_TEXTURE_LEAVES,
    BLOCK_TEXTURE_CACTUS_TOP,
    BLOCK_TEXTURE_CACTUS_SIDE,
    BLOCK_TEXTURE_SIZE
} BlockTexture;

typedef enum BlockType {
    BLOCK_TYPE_AIR,
    BLOCK_TYPE_GRASS,
    BLOCK_TYPE_DIRT,
    BLOCK_TYPE_WATER,
    BLOCK_TYPE_SAND,
    BLOCK_TYPE_STONE,
    BLOCK_TYPE_COAL,
    BLOCK_TYPE_GOLD,
    BLOCK_TYPE_TRUNK,
    BLOCK_TYPE_LEAVES,
    BLOCK_TYPE_CACTUS,
    BLOCK_TYPE_SIZE
} BlockType;

extern BlockTexture BLOCK_TEXTURE_FACES[BLOCK_TYPE_SIZE][6];

extern float BLOCK_VERTICES[];

extern GLuint block_vertex_array;

extern GLuint block_vertex_buffer;

void block_init();

void block_free();

#endif
