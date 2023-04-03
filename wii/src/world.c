#include "world.h"

#include <math.h>
#include <stdlib.h>

static lwp_t worldWorkerThread;

void world_init(World *world, Camera *camera) {
    world->camera = camera;
    world->chunks = malloc(WORLD_CHUNK_BUFFER_SIZE * sizeof(Chunk));
    world->chunksSize = 0;

    // Create worker thread
    LWP_CreateThread(&worldWorkerThread, world_worker_thread, world, NULL, 0, 1);
}

void *world_worker_thread(void *arg) {
    World *world = arg;
    for (;;) {
        // Generate and bake chunks around camera
        int32_t cameraChunkX = round(world->camera->position.x / CHUNK_SIZE);
        int32_t cameraChunkY = round(world->camera->position.y / CHUNK_SIZE);
        int32_t cameraChunkZ = round(world->camera->position.z / CHUNK_SIZE);
        for (int32_t chunkZ = cameraChunkZ - WORLD_RENDER_DIST; chunkZ < cameraChunkZ + WORLD_RENDER_DIST; chunkZ++) {
            for (int32_t chunkY = cameraChunkY - WORLD_RENDER_DIST; chunkY < cameraChunkY + WORLD_RENDER_DIST;
                 chunkY++) {
                for (int32_t chunkX = cameraChunkX - WORLD_RENDER_DIST; chunkX < cameraChunkX + WORLD_RENDER_DIST;
                     chunkX++) {
                    Chunk *chunk = world_find_chunk(world, chunkX, chunkY, chunkZ);
                    if (chunk == NULL && world->chunksSize < WORLD_CHUNK_BUFFER_SIZE) {
                        chunk = &world->chunks[world->chunksSize++];
                        chunk_generate(chunk, chunkX, chunkY, chunkZ);
                    }
                    if (chunk != NULL && !chunk->baked) {
                        chunk_bake(chunk, world);
                    }
                }
            }
        }

        // When done yield worker thread
        LWP_YieldThread();
    }
    return NULL;
}

Chunk *world_find_chunk(World *world, int32_t chunkX, int32_t chunkY, int32_t chunkZ) {
    for (size_t i = 0; i < world->chunksSize; i++) {
        Chunk *chunk = &world->chunks[i];
        if (chunk->x == chunkX && chunk->y == chunkY && chunk->z == chunkZ) {
            return chunk;
        }
    }
    return NULL;
}

void world_render(World *world) {
    // Enable fog at chunk render distance edge
    GXColor fogColor = {176, 232, 252, 255};
    GX_SetFog(GX_FOG_LIN, CHUNK_SIZE * (WORLD_RENDER_DIST - 1), CHUNK_SIZE * WORLD_RENDER_DIST, 0.1, 1000, fogColor);

    // Render chunks around camera
    int32_t cameraChunkX = round(world->camera->position.x / CHUNK_SIZE);
    int32_t cameraChunkY = round(world->camera->position.y / CHUNK_SIZE);
    int32_t cameraChunkZ = round(world->camera->position.z / CHUNK_SIZE);
    chunk_render_begin(world->camera);
    for (int32_t chunkZ = cameraChunkZ - WORLD_RENDER_DIST; chunkZ < cameraChunkZ + WORLD_RENDER_DIST; chunkZ++) {
        for (int32_t chunkY = cameraChunkY - WORLD_RENDER_DIST; chunkY < cameraChunkY + WORLD_RENDER_DIST; chunkY++) {
            for (int32_t chunkX = cameraChunkX - WORLD_RENDER_DIST; chunkX < cameraChunkX + WORLD_RENDER_DIST;
                 chunkX++) {
                Chunk *chunk = world_find_chunk(world, chunkX, chunkY, chunkZ);
                if (chunk != NULL && chunk->baked && chunk_in_camera(world, chunk)) {
                    chunk_render(chunk);
                }
            }
        }
    }

    // Disable fog
    GX_SetFog(GX_FOG_NONE, 0.1, 1.0, 0.0, 1.0, (GXColor){0, 0, 0, 255});
}
