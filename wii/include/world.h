#pragma once

#include "camera.h"
typedef struct World World;  // Forward define
#include "chunk.h"

#define WORLD_CHUNK_BUFFER_SIZE 2048
#define WORLD_RENDER_DIST 4

struct World {
    Camera *camera;
    Chunk *chunks;
    size_t chunksSize;
};

void world_init(World *world, Camera *camera);

void *world_worker_thread(void *arg);

Chunk *world_find_chunk(World *world, int32_t chunkX, int32_t chunkY, int32_t chunkZ);

void world_render(World *world);
