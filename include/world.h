// PlaatCraft - World Header

#ifndef WORLD_H
#define WORLD_H

#include <stdint.h>
#include "tinycthread/tinycthread.h"
#include "camera.h"
#include "shaders/block_shader.h"
#include "textures/texture_atlas.h"

typedef struct World World; // Fix circle dependancy
#include "chunk.h"
#include "database.h"

typedef enum WorldRequestType {
    WORLD_REQUEST_TYPE_CHUNK_NEW = 0,
    WORLD_REQUEST_TYPE_CHUNK_UPDATE
} WorldRequestType;

typedef struct WorldRequest {
    WorldRequestType type;
    union {
        Chunk* chunk_pointer;

        struct {
            int x;
            int y;
            int z;
        } chunk_position;
    } arguments;
} WorldRequest;

struct World {
    int64_t seed;
    bool is_wireframed;
    bool is_flat_shaded;
    int render_distance;

    Database *database;

    Chunk* chunk_cache[2048];
    int chunk_cache_start;
    mtx_t chunk_cache_lock;

    WorldRequest* request_queue[2048];
    int request_queue_size;
    mtx_t request_queue_lock;

    thrd_t worker_threads[2];
    bool worker_running;
    mtx_t worker_running_lock;
};

World* world_new(Camera* camera);

void world_add_chunk_to_cache(World* world, Chunk* chunk);

Chunk* world_get_chunk(World* world, int chunk_x, int chunk_y, int chunk_z);

Chunk* world_request_chunk(World* world, int chunk_x, int chunk_y, int chunk_z);

void world_request_chunk_update(World* world, Chunk* chunk);

int world_render(World* world, Camera* camera, BlockShader* block_shader, TextureAtlas* blocks_texture_atlas);

void world_free(World* world, Camera* camera);

int world_worker_thread(void* argument);

#endif
