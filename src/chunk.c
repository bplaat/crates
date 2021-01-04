// PlaatCraft - Chunk

#include "chunk.h"
#include <stdlib.h>
#include "log.h"
#include "geometry/block.h"

static int SEED;

static int hash[] = {208,34,231,213,32,248,233,56,161,78,24,140,71,48,140,254,245,255,247,247,40,
                     185,248,251,245,28,124,204,204,76,36,1,107,28,234,163,202,224,245,128,167,204,
                     9,92,217,54,239,174,173,102,193,189,190,121,100,108,167,44,43,77,180,204,8,81,
                     70,223,11,38,24,254,210,210,177,32,81,195,243,125,8,169,112,32,97,53,195,13,
                     203,9,47,104,125,117,114,124,165,203,181,235,193,206,70,180,174,0,167,181,41,
                     164,30,116,127,198,245,146,87,224,149,206,57,4,192,210,65,210,129,240,178,105,
                     228,108,245,148,140,40,35,195,38,58,65,207,215,253,65,85,208,76,62,3,237,55,89,
                     232,50,217,64,244,157,199,121,252,90,17,212,203,149,152,140,187,234,177,73,174,
                     193,100,192,143,97,53,145,135,19,103,13,90,135,151,199,91,239,247,33,39,145,
                     101,120,99,3,186,86,99,41,237,203,111,79,220,135,158,42,30,154,120,67,87,167,
                     135,176,183,191,253,115,184,21,233,58,129,233,142,39,128,211,118,137,139,255,
                     114,20,218,113,154,27,127,246,250,1,8,198,250,209,92,222,173,21,88,102,219};

int noise2(int x, int y)
{
    int tmp = hash[(y + SEED) % 256];
    return hash[(tmp + x) % 256];
}

float lin_inter(float x, float y, float s)
{
    return x + s * (y-x);
}

float smooth_inter(float x, float y, float s)
{
    return lin_inter(x, y, s * s * (3-2*s));
}

float noise2d(float x, float y)
{
    int x_int = x;
    int y_int = y;
    float x_frac = x - x_int;
    float y_frac = y - y_int;
    int s = noise2(x_int, y_int);
    int t = noise2(x_int+1, y_int);
    int u = noise2(x_int, y_int+1);
    int v = noise2(x_int+1, y_int+1);
    float low = smooth_inter(s, t, x_frac);
    float high = smooth_inter(u, v, x_frac);
    return smooth_inter(low, high, y_frac);
}

float perlin2d(float x, float y, float freq, int depth)
{
    float xa = x*freq;
    float ya = y*freq;
    float amp = 1.0;
    float fin = 0;
    float div = 0.0;

    int i;
    for(i=0; i<depth; i++)
    {
        div += 256 * amp;
        fin += noise2d(xa, ya) * amp;
        amp /= 2;
        xa *= 2;
        ya *= 2;
    }

    return fin/div;
}


BlockType generate_block(int x, int y, int z) {
    int height = perlin2d((float)(x + 10000) / 16, (float)(z + 10000) / 16, 1, 1) * 16;

    int weatness = perlin2d((float)(x + 100000000) / 16, (float)(z + 100000000) / 16, 1, 1) * 3;

    int sea_level = 0;
    if (y <= sea_level) {
        return BLOCK_TYPE_WATER;
    }

    if (weatness > 0.5) {
        if (y == height) {
            return BLOCK_TYPE_GRASS;
        }

        if (y >= -8 + height && y < height) {
            return BLOCK_TYPE_DIRT;
        }
    } else {
        if (y >= -8 + height && y <= height) {
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
    // log_info("Chunk create %d %d %d", chunk_x, chunk_y, chunk_z);

    Chunk *chunk = malloc(sizeof(Chunk));
    chunk->x = chunk_x;
    chunk->y = chunk_y;
    chunk->z = chunk_z;
    chunk->is_changed = true;
    chunk->data = malloc(CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE);

    srand(chunk_x + chunk_y + chunk_z);

    for (int block_z = 0; block_z < CHUNK_SIZE; block_z++) {
        for (int block_y = 0; block_y < CHUNK_SIZE; block_y++) {
            for (int block_x = 0; block_x < CHUNK_SIZE; block_x++) {
                int block_index = block_z * CHUNK_SIZE * CHUNK_SIZE + block_y * CHUNK_SIZE + block_x;

                // Change block
                BlockType blockType = generate_block(chunk_x * CHUNK_SIZE + block_x, chunk_y * CHUNK_SIZE + block_y, chunk_z * CHUNK_SIZE + block_z);
                if (blockType == BLOCK_TYPE_AIR) {
                    if (chunk->data[block_index] != BLOCK_TYPE_TRUNK && chunk->data[block_index] != BLOCK_TYPE_LEAVES && chunk->data[block_index] != BLOCK_TYPE_CACTUS) {
                        chunk->data[block_index] = BLOCK_TYPE_AIR;
                    }
                } else {
                    chunk->data[block_index] = blockType;
                }

                // Place tree
                if (
                    block_x >= 2 && block_z >= 2 && block_x < CHUNK_SIZE - 3 && block_z < CHUNK_SIZE - 3 && block_y < CHUNK_SIZE - 7 &&
                    blockType == BLOCK_TYPE_GRASS && (rand() % 100) == 0
                ) {
                    for (int tree_y = 1; tree_y < 6; tree_y++) {
                        if (tree_y >= 3) {
                            for (int leave_z = -1; leave_z <= 1; leave_z++) {
                                for (int leave_x = -1; leave_x <= 1; leave_x++) {
                                    chunk->data[(block_z + leave_z) * CHUNK_SIZE * CHUNK_SIZE + (block_y + tree_y) * CHUNK_SIZE + (block_x + leave_x)] =
                                        leave_z == 0 && leave_x == 0 && tree_y != 5 ? BLOCK_TYPE_TRUNK : BLOCK_TYPE_LEAVES;
                                }
                            }
                        } else {
                            chunk->data[block_z * CHUNK_SIZE * CHUNK_SIZE + (block_y + tree_y) * CHUNK_SIZE + block_x] = BLOCK_TYPE_TRUNK;
                        }
                    }
                }

                // Place cactus
                if (block_y < CHUNK_SIZE - 6 && blockType == BLOCK_TYPE_SAND && (rand() % 200) == 0) {
                    for (int cactus_y = 1; cactus_y < 5; cactus_y++) {
                        chunk->data[block_z * CHUNK_SIZE * CHUNK_SIZE + (block_y + cactus_y) * CHUNK_SIZE + block_x] = BLOCK_TYPE_CACTUS;
                    }
                }
            }
        }
    }

    return chunk;
}

void chunk_update_data(Chunk* chunk, World* world) {
    if (chunk->is_changed) {
        chunk->is_changed = false;

        // log_info("Chunk update data %d %d %d", chunk->x, chunk->y, chunk->z);

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
