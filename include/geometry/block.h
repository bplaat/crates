// PlaatCraft - Block Geometry Header

#ifndef BLOCK_H
#define BLOCK_H

#include "glad/glad.h"

typedef enum BlockTexture {
    BLOCK_TEXTURE_GRASS_TOP = 0,
    BLOCK_TEXTURE_GRASS_SIDE,
    BLOCK_TEXTURE_DIRT,
    BLOCK_TEXTURE_WATER,

    BLOCK_TEXTURE_SAND,
    BLOCK_TEXTURE_STONE,
    BLOCK_TEXTURE_COAL,
    BLOCK_TEXTURE_GOLD,

    BLOCK_TEXTURE_OAK_TRUNK_TOP,
    BLOCK_TEXTURE_OAK_TRUNK_SIDE,
    BLOCK_TEXTURE_BEECH_TRUNK_TOP,
    BLOCK_TEXTURE_BEECH_TRUNK_SIDE,

    BLOCK_TEXTURE_GREEN_LEAVES,
    BLOCK_TEXTURE_ORANGE_LEAVES,
    BLOCK_TEXTURE_CACTUS_TOP,
    BLOCK_TEXTURE_CACTUS_SIDE,

    BLOCK_TEXTURE_SIZE
} BlockTexture;

typedef enum BlockType {
    BLOCK_TYPE_AIR = 0,

    BLOCK_TYPE_GRASS,
    BLOCK_TYPE_DIRT,
    BLOCK_TYPE_WATER,

    BLOCK_TYPE_SAND_TOP,
    BLOCK_TYPE_SAND,
    BLOCK_TYPE_STONE,
    BLOCK_TYPE_COAL,
    BLOCK_TYPE_GOLD,

    BLOCK_TYPE_OAK_TRUNK,
    BLOCK_TYPE_BEECH_TRUNK,

    BLOCK_TYPE_GREEN_LEAVES,
    BLOCK_TYPE_ORANGE_LEAVES,
    BLOCK_TYPE_CACTUS,

    BLOCK_TYPE_SIZE
} BlockType;

extern char* BLOCK_TEXTURE_NAMES[BLOCK_TYPE_SIZE];

extern BlockTexture BLOCK_TEXTURE_FACES[BLOCK_TYPE_SIZE][6];

#define BLOCK_VERTICES_COUNT 36

extern float BLOCK_VERTICES[];

typedef struct Block {
    GLuint vertex_array;
    GLuint vertex_buffer;
} Block;

Block* block_new(void);

void block_enable(Block* block);

void block_disable(Block* block);

void block_free(Block* block);

#endif
