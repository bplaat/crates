// PlaatCraft - Chunk

#include "chunk.h"
#include <stdlib.h>
#include <string.h>
#include "log.h"
#include "geometry/block.h"
#include "perlin/perlin.h"

BlockType chunk_generate_block(int x, int y, int z) {
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
    if ((rand() % 75) == 0) {
        return BLOCK_TYPE_COAL;
    }

    if ((rand() % 100) == 0) {
        return BLOCK_TYPE_GOLD;
    }

    return BLOCK_TYPE_STONE;
}

Chunk* chunk_new_from_generator(int chunk_x, int chunk_y, int chunk_z) {
    uint8_t* chunk_data = malloc(CHUNK_DATA_SIZE);
    memset(chunk_data, BLOCK_TYPE_AIR, CHUNK_DATA_SIZE);

    srand(chunk_x + chunk_y + chunk_z);
    for (int block_z = 0; block_z < CHUNK_SIZE; block_z++) {
        for (int block_y = 0; block_y < CHUNK_SIZE; block_y++) {
            for (int block_x = 0; block_x < CHUNK_SIZE; block_x++) {
                int block_index = block_z * CHUNK_SIZE * CHUNK_SIZE + block_y * CHUNK_SIZE + block_x;

                // Change block
                BlockType blockType = chunk_generate_block(chunk_x * CHUNK_SIZE + block_x, chunk_y * CHUNK_SIZE + block_y, chunk_z * CHUNK_SIZE + block_z);
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
                    blockType == BLOCK_TYPE_GRASS && (rand() % 75) == 0 &&
                    block_x >= 2 && block_x <= CHUNK_SIZE - 2 &&
                    block_y >= 1 && block_y <= CHUNK_SIZE - 7 &&
                    block_z >= 2 && block_z <= CHUNK_SIZE - 2
                ) {
                    int oak_type = rand() % 2 == 0;
                    int leave_type = rand() % 2 == 0;

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
                    blockType == BLOCK_TYPE_SAND_TOP && (rand() % 150) == 0 &&
                    block_y >= 1 && block_y <= CHUNK_SIZE - 7
                ) {
                    for (int cactus_y = 1; cactus_y < 5; cactus_y++) {
                        chunk_data[block_z * CHUNK_SIZE * CHUNK_SIZE + (block_y + cactus_y) * CHUNK_SIZE + block_x] = BLOCK_TYPE_CACTUS;
                    }
                }
            }
        }
    }

    return chunk_new_from_data(chunk_x, chunk_y, chunk_z, chunk_data);
}

Chunk* chunk_new_from_data(int chunk_x, int chunk_y, int chunk_z, uint8_t* chunk_data) {
    log_debug("Chunk create %d %d %d", chunk_x, chunk_y, chunk_z);

    Chunk* chunk = malloc(sizeof(Chunk));
    chunk->x = chunk_x;
    chunk->y = chunk_y;
    chunk->z = chunk_z;
    chunk->is_freed = false;
    chunk->is_changed = true;
    chunk->data = chunk_data;
    return chunk;
}

void chunk_update(Chunk* chunk, World* world) {
    if (chunk->is_freed) {
        return;
    }

    if (chunk->is_changed) {
        chunk->is_changed = false;

        log_debug("Chunk update %d %d %d", chunk->x, chunk->y, chunk->z);

        uint8_t* temp_data = malloc(CHUNK_DATA_SIZE);
        memcpy(temp_data, chunk->data, CHUNK_DATA_SIZE);

        for (int block_z = 0; block_z < CHUNK_SIZE; block_z++) {
            for (int block_y = 0; block_y < CHUNK_SIZE; block_y++) {
                for (int block_x = 0; block_x < CHUNK_SIZE; block_x++) {
                    if (chunk->is_freed) {
                        free(temp_data);
                        return;
                    }

                    int block_index = block_z * CHUNK_SIZE * CHUNK_SIZE + block_y * CHUNK_SIZE + block_x;

                    // Check air block
                    BlockType blockType = temp_data[block_index];
                    if (blockType == BLOCK_TYPE_AIR) {
                        temp_data[block_index] &= ~CHUNK_DATA_VISIBLE_BIT;
                        continue;
                    }

                    // Check above block
                    if (block_y == 0) {
                        Chunk *otherChunk = world_get_chunk(world, chunk->x, chunk->y - 1, chunk->z);
                        BlockType blockType = otherChunk->data[block_z * CHUNK_SIZE * CHUNK_SIZE + (CHUNK_SIZE - 1) * CHUNK_SIZE + block_x];
                        if (blockType == BLOCK_TYPE_AIR) {
                            temp_data[block_index] |= CHUNK_DATA_VISIBLE_BIT;
                            continue;
                        }
                    } else {
                        BlockType blockType = temp_data[block_z * CHUNK_SIZE * CHUNK_SIZE + (block_y - 1) * CHUNK_SIZE + block_x];
                        if (blockType == BLOCK_TYPE_AIR) {
                            temp_data[block_index] |= CHUNK_DATA_VISIBLE_BIT;
                            continue;
                        }
                    }

                    // Check below block
                    if (block_y == CHUNK_SIZE - 1) {
                        Chunk *otherChunk = world_get_chunk(world, chunk->x, chunk->y + 1, chunk->z);
                        BlockType blockType = otherChunk->data[block_z * CHUNK_SIZE * CHUNK_SIZE + 0 * CHUNK_SIZE + block_x];
                        if (blockType == BLOCK_TYPE_AIR) {
                            temp_data[block_index] |= CHUNK_DATA_VISIBLE_BIT;
                            continue;
                        }
                    } else {
                        BlockType blockType = temp_data[block_z * CHUNK_SIZE * CHUNK_SIZE + (block_y + 1) * CHUNK_SIZE + block_x];
                        if (blockType == BLOCK_TYPE_AIR) {
                            temp_data[block_index] |= CHUNK_DATA_VISIBLE_BIT;
                            continue;
                        }
                    }

                    // Check left block
                    if (block_x == 0) {
                        Chunk *otherChunk = world_get_chunk(world, chunk->x - 1, chunk->y, chunk->z);
                        BlockType blockType = otherChunk->data[block_z * CHUNK_SIZE * CHUNK_SIZE + block_y * CHUNK_SIZE + (CHUNK_SIZE - 1)];
                        if (blockType == BLOCK_TYPE_AIR) {
                            temp_data[block_index] |= CHUNK_DATA_VISIBLE_BIT;
                            continue;
                        }
                    } else {
                        BlockType blockType = temp_data[block_z * CHUNK_SIZE * CHUNK_SIZE + block_y * CHUNK_SIZE + (block_x - 1)];
                        if (blockType == BLOCK_TYPE_AIR) {
                            temp_data[block_index] |= CHUNK_DATA_VISIBLE_BIT;
                            continue;
                        }
                    }

                    // Check right block
                    if (block_x == CHUNK_SIZE - 1) {
                        Chunk *otherChunk = world_get_chunk(world, chunk->x + 1, chunk->y, chunk->z);
                        BlockType blockType = otherChunk->data[block_z * CHUNK_SIZE * CHUNK_SIZE + block_y * CHUNK_SIZE + 0];
                        if (blockType == BLOCK_TYPE_AIR) {
                            temp_data[block_index] |= CHUNK_DATA_VISIBLE_BIT;
                            continue;
                        }
                    } else {
                        BlockType blockType = temp_data[block_z * CHUNK_SIZE * CHUNK_SIZE + block_y * CHUNK_SIZE + (block_x + 1)];
                        if (blockType == BLOCK_TYPE_AIR) {
                            temp_data[block_index] |= CHUNK_DATA_VISIBLE_BIT;
                            continue;
                        }
                    }

                    // Check front block
                    if (block_z == 0) {
                        Chunk *otherChunk = world_get_chunk(world, chunk->x, chunk->y, chunk->z - 1);
                        BlockType blockType = otherChunk->data[(CHUNK_SIZE - 1) * CHUNK_SIZE * CHUNK_SIZE + block_y * CHUNK_SIZE + block_x];
                        if (blockType == BLOCK_TYPE_AIR) {
                            temp_data[block_index] |= CHUNK_DATA_VISIBLE_BIT;
                            continue;
                        }
                    } else {
                        BlockType blockType = temp_data[(block_z - 1) * CHUNK_SIZE * CHUNK_SIZE + block_y * CHUNK_SIZE + block_x];
                        if (blockType == BLOCK_TYPE_AIR) {
                            temp_data[block_index] |= CHUNK_DATA_VISIBLE_BIT;
                            continue;
                        }
                    }

                    // Check back block
                    if (block_z == CHUNK_SIZE - 1) {
                        Chunk *otherChunk = world_get_chunk(world, chunk->x, chunk->y, chunk->z + 1);
                        BlockType blockType = otherChunk->data[0 * CHUNK_SIZE * CHUNK_SIZE + block_y * CHUNK_SIZE + block_x];
                        if (blockType == BLOCK_TYPE_AIR) {
                            temp_data[block_index] |= CHUNK_DATA_VISIBLE_BIT;
                            continue;
                        }
                    } else {
                        BlockType blockType = temp_data[(block_z + 1) * CHUNK_SIZE * CHUNK_SIZE + block_y * CHUNK_SIZE + block_x];
                        if (blockType == BLOCK_TYPE_AIR) {
                            temp_data[block_index] |= CHUNK_DATA_VISIBLE_BIT;
                            continue;
                        }
                    }

                    // Not visible
                    temp_data[block_index] &= ~CHUNK_DATA_VISIBLE_BIT;
                }
            }
        }

        memcpy(chunk->data, temp_data, CHUNK_DATA_SIZE);
        free(temp_data);
    }
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

    log_debug("Compressed chunk is %d bytes large", compressed_size);

    return compressed_data;
}

// Decompress chunk data for a simple run length encoding
uint8_t* chunk_data_decompress(uint8_t* compressed_data) {
    int compressed_size = (compressed_data[1] << 8) | compressed_data[0];

    log_debug("Decompressed chunk is %d bytes large", compressed_size);

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
    if (!chunk->is_freed) {
        chunk->is_freed = true;
        free(chunk->data);
        free(chunk);
    }
}
