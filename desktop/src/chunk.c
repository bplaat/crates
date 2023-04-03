// PlaatCraft - Chunk

#include "chunk.h"
#include <stdlib.h>
#include <string.h>
#include "log.h"
#include "geometry/block.h"
#include "perlin/perlin.h"

BlockType chunk_generate_block(Random *random, int x, int y, int z) {
    float scale = 64;
    int max_height = 24;
    int sea_level = -5;
    int height = perlin_noise((float)x / scale, 1, (float)z / scale) * max_height;
    if (height < sea_level) height = sea_level;
    int dryness = perlin_noise((float)(x + 1000000) / scale, 1, (float)(z + 1000000) / scale) * (max_height / 2);

    // Air layer
    if (y > height) {
        return BLOCK_TYPE_AIR;
    }

    // Sea layer
    if (height == sea_level) {
        return BLOCK_TYPE_WATER;
    }

    // Grass / Dirt / Sand layer
    if (dryness < 0.5) {
        if (y == height) {
            return BLOCK_TYPE_GRASS;
        }
        else if (y >= -8 + height) {
            return BLOCK_TYPE_DIRT;
        }
    } else {
        if (y == height) {
            return BLOCK_TYPE_SAND_TOP;
        }
        else if (y >= -8 + height) {
            return BLOCK_TYPE_SAND;
        }
    }

    // Stone layer
    if (random_rand(random, 1, 75) == 1) {
        return BLOCK_TYPE_COAL;
    }

    if (random_rand(random, 1, 100) == 1) {
        return BLOCK_TYPE_GOLD;
    }

    return BLOCK_TYPE_STONE;
}

Chunk* chunk_new_from_generator(int chunk_x, int chunk_y, int chunk_z) {
    uint8_t* chunk_data = malloc(CHUNK_DATA_SIZE);
    memset(chunk_data, BLOCK_TYPE_AIR, CHUNK_DATA_SIZE);

    Random *random = random_new(chunk_x + chunk_y + chunk_z);

    for (int block_z = 0; block_z < CHUNK_SIZE; block_z++) {
        for (int block_y = 0; block_y < CHUNK_SIZE; block_y++) {
            for (int block_x = 0; block_x < CHUNK_SIZE; block_x++) {
                int block_index = block_z * CHUNK_SIZE * CHUNK_SIZE + block_y * CHUNK_SIZE + block_x;

                // Change block
                BlockType blockType = chunk_generate_block(random, chunk_x * CHUNK_SIZE + block_x, chunk_y * CHUNK_SIZE + block_y, chunk_z * CHUNK_SIZE + block_z);
                if (blockType == BLOCK_TYPE_AIR) {
                    if (
                        chunk_data[block_index] != BLOCK_TYPE_OAK_TRUNK &&
                        chunk_data[block_index] != BLOCK_TYPE_BEECH_TRUNK &&
                        chunk_data[block_index] != BLOCK_TYPE_GREEN_LEAVES &&
                        chunk_data[block_index] != BLOCK_TYPE_ORANGE_LEAVES &&
                        chunk_data[block_index] != BLOCK_TYPE_CACTUS
                    ) {
                        chunk_data[block_index] = BLOCK_TYPE_AIR;
                    }
                } else {
                    chunk_data[block_index] = blockType;
                }

                // Place trees
                if (
                    blockType == BLOCK_TYPE_GRASS && random_rand(random, 1, 75) == 1 &&
                    block_x >= 2 && block_x <= CHUNK_SIZE - 2 &&
                    block_y >= 1 && block_y <= CHUNK_SIZE - 7 &&
                    block_z >= 2 && block_z <= CHUNK_SIZE - 2
                ) {
                    bool oak_type = random_rand(random, 1, 2) == 1;
                    bool leave_type = random_rand(random, 1, 2) == 1;

                    for (int tree_y = 1; tree_y < 6; tree_y++) {
                        if (tree_y >= 3) {
                            for (int leave_z = -1; leave_z <= 1; leave_z++) {
                                for (int leave_x = -1; leave_x <= 1; leave_x++) {
                                    chunk_data[(block_z + leave_z) * CHUNK_SIZE * CHUNK_SIZE + (block_y + tree_y) * CHUNK_SIZE + (block_x + leave_x)] =
                                        leave_z == 0 && leave_x == 0 && tree_y != 5
                                            ? (oak_type ? BLOCK_TYPE_BEECH_TRUNK : BLOCK_TYPE_OAK_TRUNK)
                                            : (leave_type ? BLOCK_TYPE_GREEN_LEAVES : BLOCK_TYPE_ORANGE_LEAVES);
                                }
                            }
                        } else {
                            chunk_data[block_z * CHUNK_SIZE * CHUNK_SIZE + (block_y + tree_y) * CHUNK_SIZE + block_x] =
                                oak_type ? BLOCK_TYPE_BEECH_TRUNK : BLOCK_TYPE_OAK_TRUNK;
                        }
                    }
                }

                // Place cactuses
                if (
                    blockType == BLOCK_TYPE_SAND_TOP && random_rand(random, 1, 150) == 1 &&
                    block_y >= 1 && block_y <= CHUNK_SIZE - 7
                ) {
                    for (int cactus_y = 1; cactus_y < 5; cactus_y++) {
                        chunk_data[block_z * CHUNK_SIZE * CHUNK_SIZE + (block_y + cactus_y) * CHUNK_SIZE + block_x] = BLOCK_TYPE_CACTUS;
                    }
                }
            }
        }
    }

    random_free(random);

    return chunk_new_from_data(chunk_x, chunk_y, chunk_z, chunk_data);
}

Chunk* chunk_new_from_data(int chunk_x, int chunk_y, int chunk_z, uint8_t* chunk_data) {
    log_debug("Chunk create %d %d %d", chunk_x, chunk_y, chunk_z);

    Chunk* chunk = malloc(sizeof(Chunk));
    chunk->x = chunk_x;
    chunk->y = chunk_y;
    chunk->z = chunk_z;
    chunk->is_changed = false;
    chunk->is_lighted = false;
    chunk->is_relighted = false;
    chunk->data = chunk_data;
    mtx_init(&chunk->chunk_lock, mtx_plain);
    return chunk;
}

void chunk_update(Chunk* chunk, World* world) {
    if (chunk->is_changed || !chunk->is_lighted || !chunk->is_relighted) {
        log_debug("Chunk update %d %d %d", chunk->x, chunk->y, chunk->z);

        mtx_lock(&chunk->chunk_lock);

        chunk->is_changed = false;
        chunk->is_lighted = true;
        chunk->is_relighted = true;

        for (int block_z = 0; block_z < CHUNK_SIZE; block_z++) {
            for (int block_y = 0; block_y < CHUNK_SIZE; block_y++) {
                for (int block_x = 0; block_x < CHUNK_SIZE; block_x++) {
                    int block_index = block_z * CHUNK_SIZE * CHUNK_SIZE + block_y * CHUNK_SIZE + block_x;

                    // Check air block
                    BlockType blockType = chunk->data[block_index];
                    if (blockType == BLOCK_TYPE_AIR) {
                        chunk->data[block_index] &= ~CHUNK_DATA_VISIBLE_BIT;
                        continue;
                    }

                    // Check left block
                    if (block_x == 0) {
                        Chunk *otherChunk = world_get_chunk(world, chunk->x - 1, chunk->y, chunk->z);
                        BlockType blockType = otherChunk->data[block_z * CHUNK_SIZE * CHUNK_SIZE + block_y * CHUNK_SIZE + (CHUNK_SIZE - 1)];
                        if (blockType == BLOCK_TYPE_AIR) {
                            chunk->data[block_index] |= CHUNK_DATA_VISIBLE_BIT;
                            continue;
                        }
                    } else {
                        BlockType blockType = chunk->data[block_z * CHUNK_SIZE * CHUNK_SIZE + block_y * CHUNK_SIZE + (block_x - 1)];
                        if (blockType == BLOCK_TYPE_AIR) {
                            chunk->data[block_index] |= CHUNK_DATA_VISIBLE_BIT;
                            continue;
                        }
                    }

                    // Check right block
                    if (block_x == CHUNK_SIZE - 1) {
                        Chunk *otherChunk = world_get_chunk(world, chunk->x + 1, chunk->y, chunk->z);
                        BlockType blockType = otherChunk->data[block_z * CHUNK_SIZE * CHUNK_SIZE + block_y * CHUNK_SIZE + 0];
                        if (blockType == BLOCK_TYPE_AIR) {
                            chunk->data[block_index] |= CHUNK_DATA_VISIBLE_BIT;
                            continue;
                        }
                    } else {
                        BlockType blockType = chunk->data[block_z * CHUNK_SIZE * CHUNK_SIZE + block_y * CHUNK_SIZE + (block_x + 1)];
                        if (blockType == BLOCK_TYPE_AIR) {
                            chunk->data[block_index] |= CHUNK_DATA_VISIBLE_BIT;
                            continue;
                        }
                    }

                    // Check above block
                    if (block_y == 0) {
                        Chunk *otherChunk = world_get_chunk(world, chunk->x, chunk->y - 1, chunk->z);
                        BlockType blockType = otherChunk->data[block_z * CHUNK_SIZE * CHUNK_SIZE + (CHUNK_SIZE - 1) * CHUNK_SIZE + block_x];
                        if (blockType == BLOCK_TYPE_AIR) {
                            chunk->data[block_index] |= CHUNK_DATA_VISIBLE_BIT;
                            continue;
                        }
                    } else {
                        BlockType blockType = chunk->data[block_z * CHUNK_SIZE * CHUNK_SIZE + (block_y - 1) * CHUNK_SIZE + block_x];
                        if (blockType == BLOCK_TYPE_AIR) {
                            chunk->data[block_index] |= CHUNK_DATA_VISIBLE_BIT;
                            continue;
                        }
                    }

                    // Check below block
                    if (block_y == CHUNK_SIZE - 1) {
                        Chunk *otherChunk = world_get_chunk(world, chunk->x, chunk->y + 1, chunk->z);
                        BlockType blockType = otherChunk->data[block_z * CHUNK_SIZE * CHUNK_SIZE + 0 * CHUNK_SIZE + block_x];
                        if (blockType == BLOCK_TYPE_AIR) {
                            chunk->data[block_index] |= CHUNK_DATA_VISIBLE_BIT;
                            continue;
                        }
                    } else {
                        BlockType blockType = chunk->data[block_z * CHUNK_SIZE * CHUNK_SIZE + (block_y + 1) * CHUNK_SIZE + block_x];
                        if (blockType == BLOCK_TYPE_AIR) {
                            chunk->data[block_index] |= CHUNK_DATA_VISIBLE_BIT;
                            continue;
                        }
                    }

                    // Check front block
                    if (block_z == 0) {
                        Chunk *otherChunk = world_get_chunk(world, chunk->x, chunk->y, chunk->z - 1);
                        BlockType blockType = otherChunk->data[(CHUNK_SIZE - 1) * CHUNK_SIZE * CHUNK_SIZE + block_y * CHUNK_SIZE + block_x];
                        if (blockType == BLOCK_TYPE_AIR) {
                            chunk->data[block_index] |= CHUNK_DATA_VISIBLE_BIT;
                            continue;
                        }
                    } else {
                        BlockType blockType = chunk->data[(block_z - 1) * CHUNK_SIZE * CHUNK_SIZE + block_y * CHUNK_SIZE + block_x];
                        if (blockType == BLOCK_TYPE_AIR) {
                            chunk->data[block_index] |= CHUNK_DATA_VISIBLE_BIT;
                            continue;
                        }
                    }

                    // Check back block
                    if (block_z == CHUNK_SIZE - 1) {
                        Chunk *otherChunk = world_get_chunk(world, chunk->x, chunk->y, chunk->z + 1);
                        BlockType blockType = otherChunk->data[0 * CHUNK_SIZE * CHUNK_SIZE + block_y * CHUNK_SIZE + block_x];
                        if (blockType == BLOCK_TYPE_AIR) {
                            chunk->data[block_index] |= CHUNK_DATA_VISIBLE_BIT;
                            continue;
                        }
                    } else {
                        BlockType blockType = chunk->data[(block_z + 1) * CHUNK_SIZE * CHUNK_SIZE + block_y * CHUNK_SIZE + block_x];
                        if (blockType == BLOCK_TYPE_AIR) {
                            chunk->data[block_index] |= CHUNK_DATA_VISIBLE_BIT;
                            continue;
                        }
                    }

                    // Not visible
                    chunk->data[block_index] &= ~CHUNK_DATA_VISIBLE_BIT;
                }
            }
        }

        mtx_unlock(&chunk->chunk_lock);
    }
}

// Check if a lighted chunk is in the camera frustum
bool chunk_is_visible(Chunk* chunk, Camera* camera) {
    // Fast check if the chunk is visible via chunk corners
    float chunk_min_x = chunk->x * CHUNK_SIZE - 0.5;
    float chunk_max_x = chunk->x * CHUNK_SIZE + CHUNK_SIZE + 0.5;
    float chunk_min_y = chunk->y * CHUNK_SIZE - 0.5;
    float chunk_max_y = chunk->y * CHUNK_SIZE + CHUNK_SIZE + 0.5;
    float chunk_min_z = chunk->z * CHUNK_SIZE - 0.5;
    float chunk_max_z = chunk->z * CHUNK_SIZE + CHUNK_SIZE + 0.5;

    Vector4 chunk_corners[BLOCK_CORNERS_COUNT] = {
        { chunk_min_x, chunk_min_y, -chunk_min_z, 1 },
        { chunk_max_x, chunk_min_y, -chunk_min_z, 1 },
        { chunk_min_x, chunk_max_y, -chunk_min_z, 1 },
        { chunk_max_x, chunk_max_y, -chunk_min_z, 1 },
        { chunk_min_x, chunk_min_y, -chunk_max_z, 1 },
        { chunk_max_x, chunk_min_y, -chunk_max_z, 1 },
        { chunk_min_x, chunk_max_y, -chunk_max_z, 1 },
        { chunk_max_x, chunk_max_y, -chunk_max_z, 1 }
    };

    for (int i = 0; i < BLOCK_CORNERS_COUNT; i++) {
        vector4_mul_matrix4(&chunk_corners[i], &camera->view_matrix);
        vector4_mul_matrix4(&chunk_corners[i], &camera->projection_matrix);
        if (
            chunk_corners[i].x >= -chunk_corners[i].w && chunk_corners[i].x <= chunk_corners[i].w &&
            chunk_corners[i].y >= -chunk_corners[i].w && chunk_corners[i].y <= chunk_corners[i].w &&
            chunk_corners[i].z >= 0 && chunk_corners[i].z <= chunk_corners[i].w
        ) {
            return true;
        }
    }

    // Check for each block in the chunk if it is visible
    for (int block_z = 0; block_z < CHUNK_SIZE; block_z++) {
        for (int block_y = 0; block_y < CHUNK_SIZE; block_y++) {
            for (int block_x = 0; block_x < CHUNK_SIZE; block_x++) {
                BlockType block_type = chunk->data[block_z * CHUNK_SIZE * CHUNK_SIZE + block_y * CHUNK_SIZE + block_x];
                if ((block_type & CHUNK_DATA_VISIBLE_BIT) != 0) {

                    float block_min_x = chunk->x * CHUNK_SIZE + block_x - 0.5;
                    float block_max_x = chunk->x * CHUNK_SIZE + block_x + 0.5;
                    float block_min_y = chunk->y * CHUNK_SIZE + block_y - 0.5;
                    float block_max_y = chunk->y * CHUNK_SIZE + block_y + 0.5;
                    float block_min_z = chunk->z * CHUNK_SIZE + block_z - 0.5;
                    float block_max_z = chunk->z * CHUNK_SIZE + block_z + 0.5;

                    Vector4 block_corners[BLOCK_CORNERS_COUNT] = {
                        { block_min_x, block_min_y, -block_min_z, 1 },
                        { block_max_x, block_min_y, -block_min_z, 1 },
                        { block_min_x, block_max_y, -block_min_z, 1 },
                        { block_max_x, block_max_y, -block_min_z, 1 },
                        { block_min_x, block_min_y, -block_max_z, 1 },
                        { block_max_x, block_min_y, -block_max_z, 1 },
                        { block_min_x, block_max_y, -block_max_z, 1 },
                        { block_max_x, block_max_y, -block_max_z, 1 }
                    };

                    for (int i = 0; i < BLOCK_CORNERS_COUNT; i++) {
                        vector4_mul_matrix4(&block_corners[i], &camera->view_matrix);
                        vector4_mul_matrix4(&block_corners[i], &camera->projection_matrix);
                        if (
                            block_corners[i].x >= -block_corners[i].w && block_corners[i].x <= block_corners[i].w &&
                            block_corners[i].y >= -block_corners[i].w && block_corners[i].y <= block_corners[i].w &&
                            block_corners[i].z >= 0 && block_corners[i].z <= block_corners[i].w
                        ) {
                            return true;
                        }
                    }
                }
            }
        }
    }

    return false;
}

// Compress chunk data with a simple run length encoding
uint8_t* chunk_data_compress(uint8_t* chunk_data) {
    uint8_t* compressed_data = malloc(CHUNK_DATA_SIZE + 2);

    bool is_only_air = true;

    int compressed_size = 2;
    int position = 0;
    while (position < CHUNK_DATA_SIZE) {
        BlockType block_type = chunk_data[position++];
        block_type &= ~CHUNK_DATA_VISIBLE_BIT;

        if (block_type != BLOCK_TYPE_AIR) {
            is_only_air = false;
        }

        int repeat_count = 0;
        for (;;) {
            BlockType next_block_type = chunk_data[position];
            next_block_type &= ~CHUNK_DATA_VISIBLE_BIT;

            if (
                position < CHUNK_DATA_SIZE &&
                block_type == next_block_type &&
                repeat_count < UINT8_MAX
            ) {
                repeat_count++;
                position++;
            }
            else {
                if (repeat_count == 0) {
                    compressed_data[compressed_size++] = block_type;
                } else {
                    compressed_data[compressed_size++] = block_type | CHUNK_COMPRESSED_DATA_REPEAT_BIT;
                    compressed_data[compressed_size++] = repeat_count;
                }
                break;
            }
        }
    }

    if (is_only_air) {
        compressed_size = 2;
    }

    compressed_data[0] = compressed_size & 0xff;
    compressed_data[1] = (compressed_size >> 8) & 0xff;

    compressed_data = realloc(compressed_data, compressed_size);

    // log_debug("Compressor: compressed chunk is %d bytes large", compressed_size);

    return compressed_data;
}

// Decompress chunk data for a simple run length encoding
uint8_t* chunk_data_decompress(uint8_t* compressed_data) {
    int compressed_size = (compressed_data[1] << 8) | compressed_data[0];

    // log_debug("Decompressor: compressed chunk is %d bytes large", compressed_size);

    uint8_t* chunk_data = malloc(CHUNK_DATA_SIZE);

    if (compressed_size == 2) {
        memset(chunk_data, BLOCK_TYPE_AIR, CHUNK_DATA_SIZE);
    }
    else {
        int compressed_position = 2;
        int position = 0;
        while (compressed_position < compressed_size) {
            if ((compressed_data[compressed_position] & CHUNK_COMPRESSED_DATA_REPEAT_BIT) != 0) {
                BlockType block_type = compressed_data[compressed_position];
                block_type &= ~CHUNK_COMPRESSED_DATA_REPEAT_BIT;
                for (int i = 0; i <= compressed_data[compressed_position + 1]; i++) {
                    chunk_data[position++] = block_type;
                }
                compressed_position += 2;
            }
            else {
                chunk_data[position++] = compressed_data[compressed_position++];
            }
        }
    }

    return chunk_data;
}

void chunk_free(Chunk* chunk) {
    mtx_lock(&chunk->chunk_lock);

    free(chunk->data);

    free(chunk);

    mtx_unlock(&chunk->chunk_lock); // Use after free?
    mtx_destroy(&chunk->chunk_lock);
}
