#pragma once

#include <stdbool.h>
#include <stdint.h>

#include "camera.h"
typedef struct Chunk Chunk;  // Forward define
#include "world.h"

#define CHUNK_SIZE 16

#define CHUNK_FACE_TOP_VISIBLE (1 << 0)
#define CHUNK_FACE_LEFT_VISIBLE (1 << 1)
#define CHUNK_FACE_FRONT_VISIBLE (1 << 2)
#define CHUNK_FACE_RIGHT_VISIBLE (1 << 3)
#define CHUNK_FACE_BACK_VISIBLE (1 << 4)
#define CHUNK_FACE_BOTTOM_VISIBLE (1 << 5)

struct Chunk {
    int32_t x;
    int32_t y;
    int32_t z;
    uint8_t blocks[CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE];
    uint8_t faces[CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE];
    bool baked;
    bool visible;
};

void chunk_generate(Chunk *chunk, int32_t chunkX, int32_t chunkY, int32_t chunkZ);

void chunk_bake(Chunk *chunk, World *world);

bool chunk_in_camera(Chunk *chunk, Camera *camera);

void chunk_render_begin(Camera *camera);

void chunk_render(Chunk *chunk);
