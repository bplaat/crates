#include "chunk.h"

#include "blocks_data.h"
#include "perlin.h"
#include "random.h"

static BlockType chunk_generate_block(int32_t x, int32_t y, int32_t z) {
    const float scale = 64;
    const int32_t max_height = 24;
    const int32_t sea_level = -5;
    int32_t height = perlin_noise((float)x / scale, 1, (float)z / scale) * max_height;
    if (height < sea_level) height = sea_level;
    int32_t dryness = perlin_noise((float)(x + 1000000) / scale, 1, (float)(z + 1000000) / scale) * (max_height / 2);

    // Air layer
    if (y > height) {
        return BLOCK_AIR;
    }

    // Sea layer
    if (height == sea_level) {
        return BLOCK_WATER;
    }

    // Grass / Dirt / Sand layer
    if (dryness < 0.5) {
        if (y == height) {
            return BLOCK_GRASS;
        } else if (y >= -8 + height) {
            return BLOCK_DIRT;
        }
    } else {
        return y == height ? BLOCK_SAND_TOP : BLOCK_SAND;
    }

    if (random_rand(1, 75) == 1) {
        return random_rand(1, 2) == 1 ? BLOCK_STONE_COAL0 : BLOCK_STONE_COAL1;
    }
    if (random_rand(1, 100) == 1) {
        return random_rand(1, 2) == 1 ? BLOCK_STONE_GOLD0 : BLOCK_STONE_GOLD1;
    }
    if (random_rand(1, 150) == 1) {
        return random_rand(1, 2) == 1 ? BLOCK_STONE_DIAMOND0 : BLOCK_STONE_DIAMOND1;
    }

    // Stone layer
    return BLOCK_STONE;
}

void chunk_generate(Chunk *chunk, int32_t chunkX, int32_t chunkY, int32_t chunkZ) {
    chunk->x = chunkX;
    chunk->y = chunkY;
    chunk->z = chunkZ;
    chunk->baked = false;
    for (int32_t blockZ = 0; blockZ < CHUNK_SIZE; blockZ++) {
        for (int32_t blockY = 0; blockY < CHUNK_SIZE; blockY++) {
            for (int32_t blockX = 0; blockX < CHUNK_SIZE; blockX++) {
                int32_t blockIndex = blockZ * CHUNK_SIZE * CHUNK_SIZE + blockY * CHUNK_SIZE + blockX;

                // Change block
                BlockType blockType = chunk_generate_block(chunkX * CHUNK_SIZE + blockX, chunkY * CHUNK_SIZE + blockY,
                                                           chunkZ * CHUNK_SIZE + blockZ);
                if (blockType == BLOCK_AIR) {
                    if (chunk->blocks[blockIndex] != BLOCK_TRUNK_OAK &&
                        chunk->blocks[blockIndex] != BLOCK_TRUNK_BEECH &&
                        chunk->blocks[blockIndex] != BLOCK_LEAVES_GREEN &&
                        chunk->blocks[blockIndex] != BLOCK_LEAVES_ORANGE && chunk->blocks[blockIndex] != BLOCK_CACTUS) {
                        chunk->blocks[blockIndex] = BLOCK_AIR;
                    }
                } else {
                    chunk->blocks[blockIndex] = blockType;
                }

                // Place trees
                if (blockType == BLOCK_GRASS && random_rand(1, 75) == 1 && blockX >= 2 && blockX <= CHUNK_SIZE - 2 &&
                    blockY >= 1 && blockY <= CHUNK_SIZE - 7 && blockZ >= 2 && blockZ <= CHUNK_SIZE - 2) {
                    bool oak_type = random_rand(1, 2) == 1;
                    bool leave_type = random_rand(1, 2) == 1;

                    for (int32_t treeY = 1; treeY < 6; treeY++) {
                        if (treeY >= 3) {
                            for (int32_t treeZ = -1; treeZ <= 1; treeZ++) {
                                for (int32_t leaveX = -1; leaveX <= 1; leaveX++) {
                                    chunk->blocks[(blockZ + treeZ) * CHUNK_SIZE * CHUNK_SIZE +
                                                  (blockY + treeY) * CHUNK_SIZE + (blockX + leaveX)] =
                                        treeZ == 0 && leaveX == 0 && treeY != 5
                                            ? (oak_type ? BLOCK_TRUNK_BEECH : BLOCK_TRUNK_OAK)
                                            : (leave_type ? BLOCK_LEAVES_GREEN : BLOCK_LEAVES_ORANGE);
                                }
                            }
                        } else {
                            chunk->blocks[blockZ * CHUNK_SIZE * CHUNK_SIZE + (blockY + treeY) * CHUNK_SIZE + blockX] =
                                oak_type ? BLOCK_TRUNK_BEECH : BLOCK_TRUNK_OAK;
                        }
                    }
                }

                // Place cactuses
                if (blockType == BLOCK_SAND_TOP && random_rand(1, 150) == 1 && blockY >= 1 &&
                    blockY <= CHUNK_SIZE - 7) {
                    for (int32_t cactusY = 1; cactusY < 5; cactusY++) {
                        chunk->blocks[blockZ * CHUNK_SIZE * CHUNK_SIZE + (blockY + cactusY) * CHUNK_SIZE + blockX] =
                            BLOCK_CACTUS;
                    }
                }
            }
        }
    }
}

void chunk_bake(Chunk *chunk, World *world) {
    Chunk *leftChunk = world_find_chunk(world, chunk->x - 1, chunk->y, chunk->z);
    Chunk *rightChunk = world_find_chunk(world, chunk->x + 1, chunk->y, chunk->z);
    Chunk *topChunk = world_find_chunk(world, chunk->x, chunk->y - 1, chunk->z);
    Chunk *bottomChunk = world_find_chunk(world, chunk->x, chunk->y + 1, chunk->z);
    Chunk *frontChunk = world_find_chunk(world, chunk->x, chunk->y, chunk->z - 1);
    Chunk *backChunk = world_find_chunk(world, chunk->x, chunk->y, chunk->z + 1);
    if (leftChunk == NULL || rightChunk == NULL || topChunk == NULL || bottomChunk == NULL || frontChunk == NULL ||
        backChunk == NULL)
        return;
    chunk->visible = false;
    chunk->baked = true;
    for (int32_t blockZ = 0; blockZ < CHUNK_SIZE; blockZ++) {
        for (int32_t blockY = 0; blockY < CHUNK_SIZE; blockY++) {
            for (int32_t blockX = 0; blockX < CHUNK_SIZE; blockX++) {
                int32_t blockIndex = blockZ * CHUNK_SIZE * CHUNK_SIZE + blockY * CHUNK_SIZE + blockX;
                BlockType blockType = chunk->blocks[blockIndex];
                chunk->faces[blockIndex] = 0;
                if (blockType == BLOCK_AIR) continue;

                // Check left face
                if (blockX == 0) {
                    if (leftChunk == NULL ||
                        leftChunk->blocks[blockZ * CHUNK_SIZE * CHUNK_SIZE + blockY * CHUNK_SIZE + (CHUNK_SIZE - 1)] ==
                            BLOCK_AIR) {
                        chunk->visible = true;
                        chunk->faces[blockIndex] |= CHUNK_FACE_LEFT_VISIBLE;
                    }
                } else if (chunk->blocks[blockZ * CHUNK_SIZE * CHUNK_SIZE + blockY * CHUNK_SIZE + (blockX - 1)] ==
                           BLOCK_AIR) {
                    chunk->visible = true;
                    chunk->faces[blockIndex] |= CHUNK_FACE_LEFT_VISIBLE;
                }

                // Check right face
                if (blockX == CHUNK_SIZE - 1) {
                    if (rightChunk == NULL ||
                        rightChunk->blocks[blockZ * CHUNK_SIZE * CHUNK_SIZE + blockY * CHUNK_SIZE + 0] == BLOCK_AIR) {
                        chunk->visible = true;
                        chunk->faces[blockIndex] |= CHUNK_FACE_RIGHT_VISIBLE;
                    }
                } else if (chunk->blocks[blockZ * CHUNK_SIZE * CHUNK_SIZE + blockY * CHUNK_SIZE + (blockX + 1)] ==
                           BLOCK_AIR) {
                    chunk->visible = true;
                    chunk->faces[blockIndex] |= CHUNK_FACE_RIGHT_VISIBLE;
                }

                // Check top face
                if (blockY == 0) {
                    if (topChunk == NULL ||
                        topChunk->blocks[blockZ * CHUNK_SIZE * CHUNK_SIZE + (CHUNK_SIZE - 1) * CHUNK_SIZE + blockX] ==
                            BLOCK_AIR) {
                        chunk->visible = true;
                        chunk->faces[blockIndex] |= CHUNK_FACE_BOTTOM_VISIBLE;
                    }
                } else if (chunk->blocks[blockZ * CHUNK_SIZE * CHUNK_SIZE + (blockY - 1) * CHUNK_SIZE + blockX] ==
                           BLOCK_AIR) {
                    chunk->visible = true;
                    chunk->faces[blockIndex] |= CHUNK_FACE_BOTTOM_VISIBLE;
                }

                // Check bottom face
                if (blockY == CHUNK_SIZE - 1) {
                    if (bottomChunk == NULL ||
                        bottomChunk->blocks[blockZ * CHUNK_SIZE * CHUNK_SIZE + 0 * CHUNK_SIZE + blockX] == BLOCK_AIR) {
                        chunk->visible = true;
                        chunk->faces[blockIndex] |= CHUNK_FACE_TOP_VISIBLE;
                    }
                } else if (chunk->blocks[blockZ * CHUNK_SIZE * CHUNK_SIZE + (blockY + 1) * CHUNK_SIZE + blockX] ==
                           BLOCK_AIR) {
                    chunk->visible = true;
                    chunk->faces[blockIndex] |= CHUNK_FACE_TOP_VISIBLE;
                }

                // Check front face
                if (blockZ == 0) {
                    if (frontChunk == NULL ||
                        frontChunk->blocks[(CHUNK_SIZE - 1) * CHUNK_SIZE * CHUNK_SIZE + blockY * CHUNK_SIZE + blockX] ==
                            BLOCK_AIR) {
                        chunk->visible = true;
                        chunk->faces[blockIndex] |= CHUNK_FACE_BACK_VISIBLE;
                    }
                } else if (chunk->blocks[(blockZ - 1) * CHUNK_SIZE * CHUNK_SIZE + blockY * CHUNK_SIZE + blockX] ==
                           BLOCK_AIR) {
                    chunk->visible = true;
                    chunk->faces[blockIndex] |= CHUNK_FACE_BACK_VISIBLE;
                }

                // Check back face
                if (blockZ == CHUNK_SIZE - 1) {
                    if (backChunk == NULL ||
                        backChunk->blocks[0 * CHUNK_SIZE * CHUNK_SIZE + blockY * CHUNK_SIZE + blockX] == BLOCK_AIR) {
                        chunk->visible = true;
                        chunk->faces[blockIndex] |= CHUNK_FACE_FRONT_VISIBLE;
                    }
                } else if (chunk->blocks[(blockZ + 1) * CHUNK_SIZE * CHUNK_SIZE + blockY * CHUNK_SIZE + blockX] ==
                           BLOCK_AIR) {
                    chunk->visible = true;
                    chunk->faces[blockIndex] |= CHUNK_FACE_FRONT_VISIBLE;
                }
            }
        }
    }
}

static Mtx cameraMatrix;

bool chunk_in_camera(Chunk *chunk, Camera *camera) {
    return true;
    // float chunk_min_x = chunk->x * CHUNK_SIZE - 0.5;
    // float chunk_max_x = chunk->x * CHUNK_SIZE + CHUNK_SIZE + 0.5;
    // float chunk_min_y = chunk->y * CHUNK_SIZE - 0.5;
    // float chunk_max_y = chunk->y * CHUNK_SIZE + CHUNK_SIZE + 0.5;
    // float chunk_min_z = chunk->z * CHUNK_SIZE - 0.5;
    // float chunk_max_z = chunk->z * CHUNK_SIZE + CHUNK_SIZE + 0.5;

    // guVector chunkCorners[] = {
    //     {chunk_min_x, chunk_min_y, -chunk_min_z}, {chunk_max_x, chunk_min_y, -chunk_min_z},
    //     {chunk_min_x, chunk_max_y, -chunk_min_z}, {chunk_max_x, chunk_max_y, -chunk_min_z},
    //     {chunk_min_x, chunk_min_y, -chunk_max_z}, {chunk_max_x, chunk_min_y, -chunk_max_z},
    //     {chunk_min_x, chunk_max_y, -chunk_max_z}, {chunk_max_x, chunk_max_y, -chunk_max_z},
    // };

    // GXRModeObj *rmode = VIDEO_GetPreferredMode(NULL);
    // Mtx44 perspectiveMatrix;
    // guPerspective(perspectiveMatrix, 45, (float)rmode->viWidth / (float)rmode->viHeight, 0.1, 1000);

    // for (size_t i = 0; i < sizeof(chunkCorners) / sizeof(guVector); i++) {
    //     guVecMultiply(cameraMatrix, &chunkCorners[i], &chunkCorners[i]);
    //     guVecMultiply(perspectiveMatrix, &chunkCorners[i], &chunkCorners[i]);
    //     if (chunkCorners[i].x >= -1 && chunkCorners[i].x <= 1 &&
    //         chunkCorners[i].y >= -1 && chunkCorners[i].y <= 1) {
    //         return true;
    //     }
    // }
    // return false;
}

// clang-format off
static float block_vertexs[] ATTRIBUTE_ALIGN(32) = {
    -0.5, 0.5, -0.5,
    0.5, 0.5, -0.5,
    -0.5, -0.5, -0.5,
    0.5, -0.5, -0.5,

    -0.5, 0.5, 0.5,
    0.5, 0.5, 0.5,
    -0.5, -0.5, 0.5,
    0.5, -0.5, 0.5,
};
static uint8_t block_colors[] ATTRIBUTE_ALIGN(32) = {
    255, 255, 255, 255,
    230, 230, 230, 255,
    179, 179, 179, 255,
    128, 128, 128, 255,
};
static float block_texcoords[] ATTRIBUTE_ALIGN(32) = {
    0, 0,
    0, 1,
    1, 0,
    1, 1
};
// clang-format on

void chunk_render_begin(Camera *camera) {
    // Init buffers
    GX_ClearVtxDesc();
    GX_SetVtxDesc(GX_VA_POS, GX_INDEX8);
    GX_SetVtxAttrFmt(GX_VTXFMT0, GX_VA_POS, GX_POS_XYZ, GX_F32, 0);
    GX_SetArray(GX_VA_POS, block_vertexs, 3 * sizeof(float));

    GX_SetVtxDesc(GX_VA_CLR0, GX_INDEX8);
    GX_SetVtxAttrFmt(GX_VTXFMT0, GX_VA_CLR0, GX_CLR_RGBA, GX_RGB8, 0);
    GX_SetArray(GX_VA_CLR0, block_colors, 4 * sizeof(uint8_t));

    GX_SetVtxDesc(GX_VA_TEX0, GX_INDEX8);
    GX_SetVtxAttrFmt(GX_VTXFMT0, GX_VA_TEX0, GX_TEX_ST, GX_F32, 0);
    GX_SetArray(GX_VA_TEX0, block_texcoords, 2 * sizeof(float));

    GX_SetNumChans(1);
    GX_SetNumTexGens(1);
    GX_SetTexCoordGen(GX_TEXCOORD0, GX_TG_MTX2x4, GX_TG_TEX0, GX_IDENTITY);
    GX_SetTevOrder(GX_TEVSTAGE0, GX_TEXCOORD0, GX_TEXMAP0, GX_COLOR0A0);
    GX_SetTevOp(GX_TEVSTAGE0, GX_MODULATE);

    // Calculate camera matrix
    guMtxRotRad(cameraMatrix, 'x', -camera->pitch);
    Mtx tempMatrix;
    guMtxRotRad(tempMatrix, 'y', -camera->yaw);
    guMtxConcat(cameraMatrix, tempMatrix, cameraMatrix);
    guMtxTrans(tempMatrix, -camera->position.x, -camera->position.y, -camera->position.z);
    guMtxConcat(cameraMatrix, tempMatrix, cameraMatrix);
}

void chunk_render(Chunk *chunk) {
    if (!chunk->visible) return;
    for (int32_t blockZ = 0; blockZ < CHUNK_SIZE; blockZ++) {
        for (int32_t blockY = 0; blockY < CHUNK_SIZE; blockY++) {
            for (int32_t blockX = 0; blockX < CHUNK_SIZE; blockX++) {
                int32_t blockIndex = blockZ * CHUNK_SIZE * CHUNK_SIZE + blockY * CHUNK_SIZE + blockX;
                if (chunk->faces[blockIndex] == 0) continue;

                // Get block info
                Block block = blocks[chunk->blocks[blockIndex]];

                // Calculate block matrix
                Mtx blockMatrix;
                guMtxTrans(blockMatrix, chunk->x * CHUNK_SIZE + blockX, chunk->y * CHUNK_SIZE + blockY,
                           chunk->z * CHUNK_SIZE + blockZ);
                guMtxConcat(cameraMatrix, blockMatrix, blockMatrix);
                GX_LoadPosMtxImm(blockMatrix, GX_PNMTX0);

                // Draw block faces
                if (chunk->faces[blockIndex] & CHUNK_FACE_TOP_VISIBLE) {
                    GX_LoadTexObj(&block_textures[block.textures[0]].texture, GX_TEXMAP0);
                    GX_Begin(GX_QUADS, GX_VTXFMT0, 4);
                    GX_Position1x8(4);
                    GX_Color1x8(0);
                    GX_TexCoord1x8(1);
                    GX_Position1x8(0);
                    GX_Color1x8(0);
                    GX_TexCoord1x8(0);
                    GX_Position1x8(1);
                    GX_Color1x8(0);
                    GX_TexCoord1x8(2);
                    GX_Position1x8(5);
                    GX_Color1x8(0);
                    GX_TexCoord1x8(3);
                    GX_End();
                }

                if (chunk->faces[blockIndex] & CHUNK_FACE_LEFT_VISIBLE) {
                    GX_LoadTexObj(&block_textures[block.textures[1]].texture, GX_TEXMAP0);
                    GX_Begin(GX_QUADS, GX_VTXFMT0, 4);
                    GX_Position1x8(0);
                    GX_Color1x8(1);
                    GX_TexCoord1x8(0);
                    GX_Position1x8(4);
                    GX_Color1x8(1);
                    GX_TexCoord1x8(2);
                    GX_Position1x8(6);
                    GX_Color1x8(1);
                    GX_TexCoord1x8(3);
                    GX_Position1x8(2);
                    GX_Color1x8(1);
                    GX_TexCoord1x8(1);
                    GX_End();
                }

                if (chunk->faces[blockIndex] & CHUNK_FACE_FRONT_VISIBLE) {
                    GX_LoadTexObj(&block_textures[block.textures[2]].texture, GX_TEXMAP0);
                    GX_Begin(GX_QUADS, GX_VTXFMT0, 4);
                    GX_Position1x8(4);
                    GX_Color1x8(1);
                    GX_TexCoord1x8(0);
                    GX_Position1x8(5);
                    GX_Color1x8(1);
                    GX_TexCoord1x8(2);
                    GX_Position1x8(7);
                    GX_Color1x8(1);
                    GX_TexCoord1x8(3);
                    GX_Position1x8(6);
                    GX_Color1x8(1);
                    GX_TexCoord1x8(1);
                    GX_End();
                }

                if (chunk->faces[blockIndex] & CHUNK_FACE_RIGHT_VISIBLE) {
                    GX_LoadTexObj(&block_textures[block.textures[3]].texture, GX_TEXMAP0);
                    GX_Begin(GX_QUADS, GX_VTXFMT0, 4);
                    GX_Position1x8(3);
                    GX_Color1x8(2);
                    GX_TexCoord1x8(3);
                    GX_Position1x8(7);
                    GX_Color1x8(2);
                    GX_TexCoord1x8(1);
                    GX_Position1x8(5);
                    GX_Color1x8(2);
                    GX_TexCoord1x8(0);
                    GX_Position1x8(1);
                    GX_Color1x8(2);
                    GX_TexCoord1x8(2);
                    GX_End();
                }

                if (chunk->faces[blockIndex] & CHUNK_FACE_BACK_VISIBLE) {
                    GX_LoadTexObj(&block_textures[block.textures[4]].texture, GX_TEXMAP0);
                    GX_Begin(GX_QUADS, GX_VTXFMT0, 4);
                    GX_Position1x8(2);
                    GX_Color1x8(2);
                    GX_TexCoord1x8(3);
                    GX_Position1x8(3);
                    GX_Color1x8(2);
                    GX_TexCoord1x8(1);
                    GX_Position1x8(1);
                    GX_Color1x8(2);
                    GX_TexCoord1x8(0);
                    GX_Position1x8(0);
                    GX_Color1x8(2);
                    GX_TexCoord1x8(2);
                    GX_End();
                }

                if (chunk->faces[blockIndex] & CHUNK_FACE_BOTTOM_VISIBLE) {
                    GX_LoadTexObj(&block_textures[block.textures[5]].texture, GX_TEXMAP0);
                    GX_Begin(GX_QUADS, GX_VTXFMT0, 4);
                    GX_Position1x8(6);
                    GX_Color1x8(3);
                    GX_TexCoord1x8(3);
                    GX_Position1x8(7);
                    GX_Color1x8(3);
                    GX_TexCoord1x8(1);
                    GX_Position1x8(3);
                    GX_Color1x8(3);
                    GX_TexCoord1x8(0);
                    GX_Position1x8(2);
                    GX_Color1x8(3);
                    GX_TexCoord1x8(2);
                    GX_End();
                }
            }
        }
    }
}
