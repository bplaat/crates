// PlaatCraft - Chunk Header

#ifndef CHUNK_H
#define CHUNK_H

#include <stdbool.h>
#include <stdint.h>
#include "config.h"
#include "tinycthread/tinycthread.h"

#define CHUNK_DATA_SIZE (CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE)
#define CHUNK_DATA_VISIBLE_BIT (1 << 7)
#define CHUNK_COMPRESSED_DATA_REPEAT_BIT (1 << 7)

typedef struct Chunk {
    int x;
    int y;
    int z;
    bool is_lighted;
    bool is_changed;
    uint8_t* data;
    mtx_t chunk_lock;
} Chunk;

#include "world.h"

BlockType chunk_generate_block(int x, int y, int z);

Chunk* chunk_new_from_generator(int chunk_x, int chunk_y, int chunk_z);

Chunk* chunk_new_from_data(int chunk_x, int chunk_y, int chunk_z, uint8_t* chunk_data);

void chunk_update(Chunk* chunk, World* world);

uint8_t* chunk_data_compress(uint8_t* chunk_data);

uint8_t* chunk_data_decompress(uint8_t* compressed_data);

void chunk_free(Chunk* chunk);

#endif
