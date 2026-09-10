#pragma once

#include <gccore.h>
#include <stdint.h>

// Textures
typedef struct BlockTexture {
    int32_t type;
    char *name;
    GXTexObj texture;
} BlockTexture;

extern BlockTexture block_textures[];

void block_textures_load(void);

// Blocks
typedef enum BlockType {
    BLOCK_AIR,
    BLOCK_GRASS,
    BLOCK_DIRT,
    BLOCK_SAND_TOP,
    BLOCK_SAND,
    BLOCK_STONE,
    BLOCK_STONE_COAL0,
    BLOCK_STONE_COAL1,
    BLOCK_STONE_GOLD0,
    BLOCK_STONE_GOLD1,
    BLOCK_STONE_DIAMOND0,
    BLOCK_STONE_DIAMOND1,
    BLOCK_WATER,
    BLOCK_TRUNK_OAK,
    BLOCK_TRUNK_BEECH,
    BLOCK_WOOD_OAK,
    BLOCK_WOOD_BEECH,
    BLOCK_LEAVES_GREEN,
    BLOCK_LEAVES_ORANGE,
    BLOCK_CACTUS,
} BlockType;

typedef struct Block {
    BlockType type;
    char *name;
    int32_t textures[6];
} Block;

extern Block blocks[];
