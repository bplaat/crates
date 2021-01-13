// PlaatCraft - Chunk Header

#ifndef CHUNK_H
#define CHUNK_H

#include <stdbool.h>
#include <stdint.h>

#define CHUNK_SIZE 16

#define CHUNK_DATA_BLOCK_TYPE ((1 << 7) - 1)
#define CHUNK_DATA_VISIBLE_BIT (1 << 7)

typedef struct Chunk {
    int x;
    int y;
    int z;
    bool is_freed;
    bool is_changed;
    uint8_t* data;
} Chunk;

#include "world.h"

BlockType chunk_generate_block(int x, int y, int z);

Chunk* chunk_new(int chunk_x, int chunk_y, int chunk_z);

void chunk_update(Chunk* chunk, World* world);

void chunk_free(Chunk* chunk);

#endif
