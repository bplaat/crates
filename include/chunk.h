// PlaatCraft - Chunk Header

#ifndef CHUNK_H
#define CHUNK_H

#include <stdbool.h>
#include <stdint.h>

#define CHUNK_SIZE 16

#define CHUNK_DATA_BLOCK_TYPE 127
#define CHUNK_DATA_VISIBLE_BIT (1 << 7)

typedef struct Chunk {
    int x;
    int y;
    int z;
    bool is_changed;
    uint8_t *data;
} Chunk;

#include "world.h"

Chunk* chunk_new(int chunk_x, int chunk_y, int chunk_z);

void chunk_update_data(Chunk* chunk, World* world);

void chunk_free(Chunk* chunk);

#endif
