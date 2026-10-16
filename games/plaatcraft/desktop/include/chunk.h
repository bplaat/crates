// PlaatCraft - Chunk Header

#ifndef CHUNK_H
#define CHUNK_H

#include <stdbool.h>
#include <stdint.h>
#include "config.h"
#include "tinycthread/tinycthread.h"
#include "camera.h"
#include "random.h"
#include "geometry/block.h"

#define CHUNK_DATA_SIZE (CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE)
#define CHUNK_DATA_VISIBLE_BIT (1 << 7)
#define CHUNK_COMPRESSED_DATA_REPEAT_BIT (1 << 7)

typedef struct Chunk {
    int x;
    int y;
    int z;
    bool is_changed;
    bool is_lighted;
    bool is_relighted;
    uint8_t* data;
    mtx_t chunk_lock;
} Chunk;

typedef struct BlockPosition {
    int chunk_x;
    int chunk_y;
    int chunk_z;
    int block_x;
    int block_y;
    int block_z;
    BlockSide block_side;
} BlockPosition;

#include "world.h"

BlockType chunk_generate_block(Random *random, int x, int y, int z);

Chunk* chunk_new_from_generator(int chunk_x, int chunk_y, int chunk_z);

Chunk* chunk_new_from_data(int chunk_x, int chunk_y, int chunk_z, uint8_t* chunk_data);

void chunk_update(Chunk* chunk, World* world);

bool chunk_is_visible(Chunk* chunk, Camera* camera);

uint8_t* chunk_data_compress(uint8_t* chunk_data);

uint8_t* chunk_data_decompress(uint8_t* compressed_data);

void chunk_free(Chunk* chunk);

#endif
