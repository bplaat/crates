// PlaatCraft - World

#include "world.h"
#include <stdlib.h>
#include <math.h>
#include "geometry/block.h"
#include "log.h"

World* world_new(int seed) {
    World* world = malloc(sizeof(World));
    world->seed = seed;

    for (size_t i = 0; i < sizeof(world->chunk_cache) / sizeof(Chunk*); i++) {
        world->chunk_cache[i] = NULL;
    }
    world->chunk_cache_start = 0;
    mtx_init(&world->chunk_cache_lock, mtx_plain);

    for (size_t i = 0; i < sizeof(world->request_queue) / sizeof(WorldRequest*); i++) {
        world->request_queue[i] = NULL;
    }
    world->request_queue_size = 0;
    mtx_init(&world->request_queue_lock, mtx_plain);

    world->worker_running = true;
    mtx_init(&world->worker_running_lock, mtx_plain);

    for (size_t i = 0; i < sizeof(world->worker_threads) / sizeof(thrd_t); i++) {
        thrd_create(&world->worker_threads[i], world_worker_thread, world);
    }

    return world;
}

Chunk* world_get_chunk(World* world, int chunk_x, int chunk_y, int chunk_z) {
    // Check if chunk is in cunk cache
    for (size_t i = 0; i < sizeof(world->chunk_cache) / sizeof(Chunk*); i++) {
        Chunk* chunk = world->chunk_cache[i];

        if (chunk == NULL) {
            break;
        }

        if (chunk->x == chunk_x && chunk->y == chunk_y && chunk->z == chunk_z) {
            return chunk;
        }
    }

    // When not found create chunk and add to cache
    mtx_lock(&world->chunk_cache_lock);

    Chunk* chunk = chunk_new(chunk_x, chunk_y, chunk_z);

    if (world->chunk_cache_start == (sizeof(world->chunk_cache) / sizeof(Chunk*))) {
        world->chunk_cache_start = 0;
    }
    if (world->chunk_cache[world->chunk_cache_start] != NULL) {
        chunk_free(world->chunk_cache[world->chunk_cache_start]);
    }
    world->chunk_cache[world->chunk_cache_start] = chunk;
    world->chunk_cache_start++;

    mtx_unlock(&world->chunk_cache_lock);

    return chunk;
}

Chunk* world_request_chunk(World* world, int chunk_x, int chunk_y, int chunk_z) {
    // Check if chunk is in cunk cache
    for (size_t i = 0; i < sizeof(world->chunk_cache) / sizeof(Chunk*); i++) {
        Chunk* chunk = world->chunk_cache[i];

        if (chunk == NULL) {
            break;
        }

        if (chunk->x == chunk_x && chunk->y == chunk_y && chunk->z == chunk_z) {
            return chunk;
        }
    }

    // Check for an pending chunk new request
    for (int i = 0; i < world->request_queue_size; i++) {
        WorldRequest* request = world->request_queue[i];
        if (
            request->type == WORLD_REQUEST_TYPE_CHUNK_NEW &&
            request->arguments.chunk_position.x == chunk_x &&
            request->arguments.chunk_position.y == chunk_y &&
            request->arguments.chunk_position.z == chunk_z
        ) {
            return NULL;
        }
    }

    // Create chunk new request and push it to the request queue
    mtx_lock(&world->request_queue_lock);
    WorldRequest* request = malloc(sizeof(WorldRequest));
    request->type = WORLD_REQUEST_TYPE_CHUNK_NEW;
    request->arguments.chunk_position.x = chunk_x;
    request->arguments.chunk_position.y = chunk_y;
    request->arguments.chunk_position.z = chunk_z;
    world->request_queue[world->request_queue_size++] = request;
    mtx_unlock(&world->request_queue_lock);
    return NULL;
}

void world_request_chunk_update(World* world, Chunk* chunk) {
    // Check for an pending chunk update request
    for (int i = 0; i < world->request_queue_size; i++) {
        WorldRequest* request = world->request_queue[i];
        if (
            request->type == WORLD_REQUEST_TYPE_CHUNK_UPDATE &&
            request->arguments.chunk_pointer == chunk
        ) {
            return;
        }
    }

    // Create chunk update request and push it to the request queue
    mtx_lock(&world->request_queue_lock);
    WorldRequest* request = malloc(sizeof(WorldRequest));
    request->type = WORLD_REQUEST_TYPE_CHUNK_UPDATE;
    request->arguments.chunk_pointer = chunk;
    world->request_queue[world->request_queue_size++] = request;
    mtx_unlock(&world->request_queue_lock);
}

void world_render(World* world, Camera* camera, BlockShader* block_shader, TextureAtlas* blocks_texture_atlas,
    int render_distance, bool is_wireframed, bool is_flat_shaded)
{
    block_shader_enable(block_shader);
    texture_atlas_enable(blocks_texture_atlas);

    if (is_wireframed) {
        glPolygonMode(GL_FRONT_AND_BACK, GL_LINE);
    }

    glUniform1i(block_shader->is_flad_shaded_uniform, is_flat_shaded);

    glUniformMatrix4fv(block_shader->projection_matrix_uniform, 1, GL_FALSE, &camera->projectionMatrix.m11);
    glUniformMatrix4fv(block_shader->camera_matrix_uniform, 1, GL_FALSE, &camera->cameraMatrix.m11);

    Matrix4 rotationMatrix;
    matrix4_rotate_x(&rotationMatrix, radians(90));

    int player_chunk_x = floor(camera->position.x / (float)CHUNK_SIZE);
    int player_chunk_y = floor(camera->position.y / (float)CHUNK_SIZE);
    int player_chunk_z = floor(camera->position.z / (float)CHUNK_SIZE);

    for (int chunk_z = player_chunk_z + render_distance; chunk_z > player_chunk_z - render_distance; chunk_z--) {
        for (int chunk_y = player_chunk_y - render_distance; chunk_y <= player_chunk_y + render_distance; chunk_y++) {
            for (int chunk_x = player_chunk_x - render_distance; chunk_x <= player_chunk_x + render_distance; chunk_x++) {

                bool chunk_is_visible = true;

                // int chunk_x_max = chunk_x < 0 ? -CHUNK_SIZE : CHUNK_SIZE;
                // int chunk_y_max = chunk_y < 0 ? -CHUNK_SIZE : CHUNK_SIZE;
                // int chunk_z_max = chunk_z < 0 ? -CHUNK_SIZE : CHUNK_SIZE;

                // Vector4 corners[8] = {
                //     { chunk_x, chunk_y, chunk_z, 1 },
                //     { chunk_x + chunk_x_max, chunk_y, chunk_z, 1 },
                //     { chunk_x, chunk_y + chunk_y_max, chunk_z, 1 },
                //     { chunk_x + chunk_x_max, chunk_y + chunk_y_max, chunk_z, 1 },

                //     { chunk_x, chunk_y, chunk_z + chunk_z_max, 1 },
                //     { chunk_x + chunk_x_max, chunk_y, chunk_z + chunk_z_max, 1 },
                //     { chunk_x, chunk_y + chunk_y_max, chunk_z + chunk_z_max, 1 },
                //     { chunk_x + chunk_x_max, chunk_y + chunk_y_max, chunk_z + chunk_z_max, 1 }
                // };

                // for (size_t i = 0; i < sizeof(corners) / sizeof(Vector4); i++) {
                //     vector4_mul(&corners[i], &camera->cameraMatrix);
                //     vector4_mul(&corners[i], &camera->projectionMatrix);

                //     #define within(a, b, c) ((a) >= (b) && (b) <= (c))
                //     if (
                //         within(-corners[i].w, corners[i].x, corners[i].w) &&
                //         within(-corners[i].w, corners[i].y, corners[i].w) &&
                //         within(0, corners[i].z, corners[i].w)
                //     ) {
                //         chunk_is_visible = true;
                //         break;
                //     }
                // }

                if (chunk_is_visible) {
                    Chunk* chunk = world_request_chunk(world, chunk_x, chunk_y, chunk_z);
                    if (chunk != NULL) {
                        if (chunk->is_changed) {
                            world_request_chunk_update(world, chunk);
                        } else {
                            for (int block_z = 0; block_z < CHUNK_SIZE; block_z++) {
                                for (int block_y = 0; block_y < CHUNK_SIZE; block_y++) {
                                    for (int block_x = 0; block_x < CHUNK_SIZE; block_x++) {
                                        uint8_t block_data = chunk->data[block_z * CHUNK_SIZE * CHUNK_SIZE + block_y * CHUNK_SIZE + block_x];
                                        if ((block_data & CHUNK_DATA_VISIBLE_BIT) != 0) {
                                            BlockType blockType = block_data & CHUNK_DATA_BLOCK_TYPE;

                                            Matrix4 modelMatrix;
                                            Vector4 blockPosition = {
                                                chunk_x * CHUNK_SIZE + block_x,
                                                chunk_y * CHUNK_SIZE + block_y,
                                                -(chunk_z * CHUNK_SIZE + block_z),
                                                1
                                            };
                                            matrix4_translate(&modelMatrix, &blockPosition);
                                            matrix4_mul(&modelMatrix, &rotationMatrix);

                                            glUniformMatrix4fv(block_shader->model_matrix_uniform, 1, GL_FALSE, &modelMatrix.m11);

                                            glUniform1iv(block_shader->texture_indexes_uniform, 6, (const GLint*)&BLOCK_TEXTURE_FACES[blockType]);

                                            glDrawArrays(GL_TRIANGLES, 0, BLOCK_VERTICES_COUNT);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    if (is_wireframed) {
        glPolygonMode(GL_FRONT_AND_BACK, GL_FILL);
    }

    texture_atlas_disable(blocks_texture_atlas);
    block_shader_disable(block_shader);
}

void world_free(World* world) {
    mtx_lock(&world->worker_running_lock);
    world->worker_running = false;
    mtx_unlock(&world->worker_running_lock);

    for (size_t i = 0; i < sizeof(world->worker_threads) / sizeof(thrd_t); i++) {
        thrd_join(world->worker_threads[i], NULL);
    }

    for (size_t i = 0; i < sizeof(world->chunk_cache) / sizeof(Chunk*); i++) {
        Chunk* chunk = world->chunk_cache[i];
        if (chunk != NULL) {
            chunk_free(chunk);
        }
    }

    free(world);
}

int world_worker_thread(void* argument) {
    World* world = (World*)argument;

    while (world->worker_running) {
        if (world->request_queue_size > 0) {
            // Get first request and remove it by shifting
            mtx_lock(&world->request_queue_lock);

            WorldRequest* request = world->request_queue[0];

            for (int i = 1; i < world->request_queue_size; i++) {
                world->request_queue[i - 1] = world->request_queue[i];
            }
            world->request_queue_size--;

            mtx_unlock(&world->request_queue_lock);

            // Create chunk
            if (request->type == WORLD_REQUEST_TYPE_CHUNK_NEW) {
                world_get_chunk(
                    world,
                    request->arguments.chunk_position.x,
                    request->arguments.chunk_position.y,
                    request->arguments.chunk_position.z
                );
            }

            // Update chunk
            if (request->type == WORLD_REQUEST_TYPE_CHUNK_UPDATE) {
                chunk_update(request->arguments.chunk_pointer, world);
            }

            // Free request
            free(request);
        } else {
            struct timespec duration = { 0, WORLD_WORKER_THREAD_UPDATE_TIMEOUT * 1000 };
            thrd_sleep(&duration, NULL);
        }
    }

    return EXIT_SUCCESS;
}
