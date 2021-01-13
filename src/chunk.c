// PlaatCraft - Chunk

#include "chunk.h"
#include <stdlib.h>
#include "log.h"
#include "geometry/block.h"
#include "perlin/perlin.h"

BlockType generate_block(int x, int y, int z) {
    float scale = 64;
    int max_height = 24;
    int height = perlin_noise((float)x / scale, 1, (float)z / scale) * max_height;
    int weatness = perlin_noise((float)(x + 1000000) / scale, 1, (float)(z + 1000000) / scale) * (max_height / 2);

    int sea_level = -5;
    if (y <= sea_level) {
        return BLOCK_TYPE_WATER;
    }

    if (weatness < 0.5) {
        if (y == height) {
            return BLOCK_TYPE_GRASS;
        }

        if (y >= -8 + height && y < height) {
            return BLOCK_TYPE_DIRT;
        }
    } else {
        if (y == height) {
            return BLOCK_TYPE_SAND_TOP;
        }

        if (y >= -8 + height && y < height) {
            return BLOCK_TYPE_SAND;
        }
    }

    if (y >= -64 + height && y < -8 + height) {
        if ((rand() % 10) == 0) {
            return BLOCK_TYPE_COAL;
        }

        if ((rand() % 20) == 0) {
            return BLOCK_TYPE_GOLD;
        }

        return BLOCK_TYPE_STONE;
    }

    return BLOCK_TYPE_AIR;
}


Chunk* chunk_new(int chunk_x, int chunk_y, int chunk_z) {
    log_debug("Chunk create %d %d %d", chunk_x, chunk_y, chunk_z);

    Chunk* chunk = malloc(sizeof(Chunk));
    chunk->x = chunk_x;
    chunk->y = chunk_y;
    chunk->z = chunk_z;
    chunk->is_changed = true;

    chunk->data = malloc(CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE);
    for (int i = 0; i < CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE; i++) {
        chunk->data[i] = BLOCK_TYPE_AIR;
    }

    srand(chunk_x + chunk_y + chunk_z);

    for (int block_z = 0; block_z < CHUNK_SIZE; block_z++) {
        for (int block_y = 0; block_y < CHUNK_SIZE; block_y++) {
            for (int block_x = 0; block_x < CHUNK_SIZE; block_x++) {
                int block_index = block_z * CHUNK_SIZE * CHUNK_SIZE + block_y * CHUNK_SIZE + block_x;

                // Change block
                BlockType blockType = generate_block(chunk_x * CHUNK_SIZE + block_x, chunk_y * CHUNK_SIZE + block_y, chunk_z * CHUNK_SIZE + block_z);
                if (blockType == BLOCK_TYPE_AIR) {
                    if (
                        chunk->data[block_index] != BLOCK_TYPE_OAK_TRUNK &&
                        chunk->data[block_index] != BLOCK_TYPE_BEECH_TRUNK &&
                        chunk->data[block_index] != BLOCK_TYPE_GREEN_LEAVES &&
                        chunk->data[block_index] != BLOCK_TYPE_ORANGE_LEAVES &&
                        chunk->data[block_index] != BLOCK_TYPE_CACTUS
                    ) {
                        chunk->data[block_index] = BLOCK_TYPE_AIR;
                    }
                } else {
                    chunk->data[block_index] = blockType;
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
                                    chunk->data[(block_z + leave_z) * CHUNK_SIZE * CHUNK_SIZE + (block_y + tree_y) * CHUNK_SIZE + (block_x + leave_x)] =
                                        leave_z == 0 && leave_x == 0 && tree_y != 5
                                            ? (oak_type ? BLOCK_TYPE_BEECH_TRUNK : BLOCK_TYPE_OAK_TRUNK)
                                            : (leave_type ? BLOCK_TYPE_GREEN_LEAVES : BLOCK_TYPE_ORANGE_LEAVES);
                                }
                            }
                        } else {
                            chunk->data[block_z * CHUNK_SIZE * CHUNK_SIZE + (block_y + tree_y) * CHUNK_SIZE + block_x] =
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
                        chunk->data[block_z * CHUNK_SIZE * CHUNK_SIZE + (block_y + cactus_y) * CHUNK_SIZE + block_x] = BLOCK_TYPE_CACTUS;
                    }
                }
            }
        }
    }

    return chunk;
}

void chunk_update(Chunk* chunk, World* world) {
    if (chunk->is_changed) {
        chunk->is_changed = false;

        log_debug("Chunk update %d %d %d", chunk->x, chunk->y, chunk->z);

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
    }
}

void chunk_free(Chunk* chunk) {
    free(chunk->data);
    free(chunk);
}
