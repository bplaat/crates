// PlaatCraft - World Header

#ifndef WORLD_H
#define WORLD_H

#include <tinycthread/tinycthread.h>
#include "camera.h"
#include "shaders/block_shader.h"
#include "textures/texture_atlas.h"

typedef struct World World;

#include "chunk.h"

typedef enum WorldRequestType {
    WORLD_REQUEST_TYPE_CHUNK_NEW,
    WORLD_REQUEST_TYPE_CHUNK_UPDATE
} WorldRequestType;

typedef struct WorldRequest {
    WorldRequestType type;
    union {
        Chunk *chunk_pointer;

        struct {
            int x;
            int y;
            int z;
        } chunk_position;
    } arguments;
} WorldRequest;

struct World {
    int seed;

    Chunk* chunk_cache[2048];
    int chunk_cache_size;
    mtx_t  chunk_cache_lock;

    WorldRequest* request_queue[2048];
    int request_queue_size;
    mtx_t  request_queue_lock;

    thrd_t worker_threads[2];
    bool worker_running;
    mtx_t  worker_running_lock;
};

World* world_new(int seed);

Chunk* world_get_chunk(World* world, int chunk_x, int chunk_y, int chunk_z);

Chunk* world_request_chunk(World* world, int chunk_x, int chunk_y, int chunk_z);

void world_request_chunk_update(World* world, Chunk* chunk);

void world_render(World* world, Camera* camera, BlockShader* blockShader, TextureAtlas* blocksTextureAtlas);

void world_free(World* world);

#define WORLD_WORKER_THREAD_UPDATE_TIMEOUT 250

int world_worker_thread(void* argument);

#endif
