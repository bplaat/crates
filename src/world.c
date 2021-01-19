// PlaatCraft - World

#include "world.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <math.h>
#include <time.h>
#include "geometry/block.h"
#include "perlin/perlin.h"
#include "random.h"
#include "log.h"

World* world_new(Camera* camera, BlockType *selected_block_type) {
    World* world = malloc(sizeof(World));
    world->is_wireframed = false;
    world->is_flat_shaded = false;
    #ifdef DEBUG
        #ifndef __WIN32__
            world->render_distance = WORLD_RENDER_DISTANCE_FAR;
        #else
            world->render_distance = WORLD_RENDER_DISTANCE_NEAR;
        #endif
    #else
        world->render_distance = WORLD_RENDER_DISTANCE_NEAR;
    #endif

    // Create database
    world->database = database_new();

    // Init chuch chache
    for (int i = 0; i < WORLD_CHUNK_CACHE_COUNT; i++) {
        world->chunk_cache[i] = NULL;
    }
    world->chunk_cache_start = 0;
    mtx_init(&world->chunk_cache_lock, mtx_plain);

    // Init request queue
    for (int i = 0; i < WORLD_REQUEST_QUEUE_COUNT; i++) {
        world->request_queue[i] = NULL;
    }
    world->request_queue_size = 0;
    mtx_init(&world->request_queue_lock, mtx_plain);

    // Get world seed
    world->seed = database_settings_get_int(world->database, "seed", 0);

    // Generate when not set
    if (world->seed == 0) {
        time_t now = time(NULL);
        struct tm *tm;
        if ((tm = localtime(&now)) == NULL) {
            log_error("Can't read / extract the local system time");
        }

        Random *random = random_new(tm->tm_hour * 3600 + tm->tm_min * 60 + tm->tm_sec);
        do {
            world->seed = random_rand(random, INT16_MIN, INT16_MAX);
        } while (world->seed == 0);
        random_free(random);

        database_settings_set_int(world->database, "seed", world->seed);
    }

    // Init perlin noise with seed
    perlin_init(world->seed);

    // Get start location right hight (y position)
    int start_x = CHUNK_SIZE / 2;
    int start_y = -1;
    int start_z = CHUNK_SIZE / 2;
    int chunk_y = -4;
    do {
        Chunk *chunk = world_get_chunk(world, 0, chunk_y, 0);
        for (int y = 0; y < CHUNK_SIZE; y++) {
            BlockType block_type = chunk->data[start_z * CHUNK_SIZE * CHUNK_SIZE + y * CHUNK_SIZE + start_x];
            block_type &= ~CHUNK_DATA_VISIBLE_BIT;
            if (block_type == BLOCK_TYPE_GRASS || block_type == BLOCK_TYPE_SAND_TOP || block_type == BLOCK_TYPE_WATER) {
                start_y = chunk_y * CHUNK_SIZE + y;
                break;
            }
        }
        chunk_y++;

        if (chunk_y == 4) {
            start_y = CHUNK_SIZE / 2;
            break;
        }
    } while (start_y != -1);

    // Get old player position
    camera->position.x = database_settings_get_float(world->database, "player_x", start_x);
    camera->position.y = database_settings_get_float(world->database, "player_y", start_y + CHUNK_SIZE / 2); // 1.5
    camera->position.z = database_settings_get_float(world->database, "player_z", start_z);

    // Get old player rotation
    camera->pitch = database_settings_get_float(world->database, "player_pitch", 0);
    camera->rotation.x = -camera->pitch;
    camera->yaw = database_settings_get_float(world->database, "player_yaw", 0);
    camera->rotation.y = -camera->yaw;
    camera_update_matrix(camera);

    // Get old player selected block
    *selected_block_type = database_settings_get_int(world->database, "player_selected_block_type", BLOCK_TYPE_BROWN_WOOD);

    // Init workers
    world->worker_running = true;
    mtx_init(&world->worker_running_lock, mtx_plain);
    for (int i = 0; i < WORLD_WORKER_THREAD_COUNT; i++) {
        thrd_create(&world->worker_threads[i], world_worker_thread, world);
    }

    return world;
}

void world_add_chunk_to_cache(World* world, Chunk* chunk) {
    mtx_lock(&world->chunk_cache_lock);
    if (world->chunk_cache_start == WORLD_CHUNK_CACHE_COUNT) {
        world->chunk_cache_start = 0;
    }
    if (world->chunk_cache[world->chunk_cache_start] != NULL) {
        chunk_free(world->chunk_cache[world->chunk_cache_start]);
    }
    world->chunk_cache[world->chunk_cache_start++] = chunk;
    mtx_unlock(&world->chunk_cache_lock);
}

Chunk* world_get_chunk(World* world, int chunk_x, int chunk_y, int chunk_z) {
    // Check if chunk is in cunk cache
    for (int i = 0; i < WORLD_CHUNK_CACHE_COUNT; i++) {
        Chunk* chunk = world->chunk_cache[i];

        if (chunk == NULL) {
            break;
        }

        if (chunk->x == chunk_x && chunk->y == chunk_y && chunk->z == chunk_z) {
            return chunk;
        }
    }

    // Select / Search chunk in database
    Chunk* chunk = database_chunks_get_chunk(world->database, chunk_x, chunk_y, chunk_z);
    if (chunk != NULL) {
        // When found add to cache
        world_add_chunk_to_cache(world, chunk);
        return chunk;
    }

    // When not found generate chunk and add to cache and add it to database
    chunk = chunk_new_from_generator(chunk_x, chunk_y, chunk_z);
    world_add_chunk_to_cache(world, chunk);
    database_chunks_set_chunk(world->database, chunk);
    return chunk;
}

Chunk* world_request_chunk(World* world, int chunk_x, int chunk_y, int chunk_z) {
    // Check if chunk is in cunk cache
    for (int i = 0; i < WORLD_CHUNK_CACHE_COUNT; i++) {
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

int world_render(World* world, Camera* camera, BlockShader* block_shader, TextureAtlas* blocks_texture_atlas) {
    block_shader_enable(block_shader);
    texture_atlas_enable(blocks_texture_atlas);

    if (world->is_wireframed) {
        glPolygonMode(GL_FRONT_AND_BACK, GL_LINE);
    }

    glUniform1i(block_shader->is_lighted_uniform, true);
    glUniform1i(block_shader->is_flad_shaded_uniform, world->is_flat_shaded);

    glUniformMatrix4fv(block_shader->projection_matrix_uniform, 1, GL_FALSE, &camera->projection_matrix.m11);
    glUniformMatrix4fv(block_shader->view_matrix_uniform, 1, GL_FALSE, &camera->view_matrix.m11);

    Matrix4 rotation_matrix;
    matrix4_rotate_x(&rotation_matrix, radians(90));

    // Loop over all rendered chunks
    int player_chunk_x = floor(camera->position.x / (float)CHUNK_SIZE);
    int player_chunk_y = floor(camera->position.y / (float)CHUNK_SIZE);
    int player_chunk_z = floor(camera->position.z / (float)CHUNK_SIZE);
    int rendered_chunks = 0;
    for (int chunk_z = player_chunk_z + world->render_distance; chunk_z > player_chunk_z - world->render_distance; chunk_z--) {
        for (int chunk_y = player_chunk_y - world->render_distance; chunk_y <= player_chunk_y + world->render_distance; chunk_y++) {
            for (int chunk_x = player_chunk_x - world->render_distance; chunk_x <= player_chunk_x + world->render_distance; chunk_x++) {
                // Get lighted chunk data
                Chunk* chunk = world_request_chunk(world, chunk_x, chunk_y, chunk_z);
                if (chunk != NULL) {
                    if (!chunk->is_lighted) {
                        world_request_chunk_update(world, chunk);
                    } else {
                        // Render the chunk when visible
                        if (chunk_is_visible(chunk, camera)) {
                            for (int block_z = 0; block_z < CHUNK_SIZE; block_z++) {
                                for (int block_y = 0; block_y < CHUNK_SIZE; block_y++) {
                                    for (int block_x = 0; block_x < CHUNK_SIZE; block_x++) {
                                        BlockType block_type = chunk->data[block_z * CHUNK_SIZE * CHUNK_SIZE + block_y * CHUNK_SIZE + block_x];
                                        if ((block_type & CHUNK_DATA_VISIBLE_BIT) != 0) {
                                            block_type &= (~CHUNK_DATA_VISIBLE_BIT);

                                            Matrix4 model_matrix;
                                            Vector4 block_position_vector = {
                                                chunk_x * CHUNK_SIZE + block_x,
                                                chunk_y * CHUNK_SIZE + block_y,
                                                -(chunk_z * CHUNK_SIZE + block_z),
                                                1
                                            };
                                            matrix4_translate(&model_matrix, &block_position_vector);
                                            matrix4_mul(&model_matrix, &rotation_matrix);
                                            glUniformMatrix4fv(block_shader->model_matrix_uniform, 1, GL_FALSE, &model_matrix.m11);

                                            glUniform1iv(block_shader->texture_indexes_uniform, 6, (const GLint*)&BLOCK_TYPE_TEXTURE_FACES[block_type]);

                                            glDrawArrays(GL_TRIANGLES, 0, BLOCK_VERTICES_COUNT);
                                        }
                                    }
                                }
                            }

                            rendered_chunks++;
                        }
                    }
                }
            }
        }
    }

    if (world->is_wireframed) {
        glPolygonMode(GL_FRONT_AND_BACK, GL_FILL);
    }

    texture_atlas_disable(blocks_texture_atlas);
    block_shader_disable(block_shader);

    return rendered_chunks;
}

BlockPosition *world_get_selected_block(World* world, Camera* camera) {
    Vector4 ray_point = camera->position;

    Matrix4 rotation_x_matrix;
    matrix4_rotate_x(&rotation_x_matrix, camera->rotation.x);

    Matrix4 rotation_y_matrix;
    matrix4_rotate_y(&rotation_y_matrix, camera->rotation.y);

    int player_chunk_x = floor(camera->position.x / (float)CHUNK_SIZE);
    int player_chunk_y = floor(camera->position.y / (float)CHUNK_SIZE);
    int player_chunk_z = floor(camera->position.z / (float)CHUNK_SIZE);

    float distance = 0;
    while (distance < CHUNK_SIZE * (world->render_distance + 1)) {
        // Check all chunks for block collision
        for (int chunk_z = player_chunk_z + world->render_distance; chunk_z > player_chunk_z - world->render_distance; chunk_z--) {
            for (int chunk_y = player_chunk_y - world->render_distance; chunk_y <= player_chunk_y + world->render_distance; chunk_y++) {
                for (int chunk_x = player_chunk_x - world->render_distance; chunk_x <= player_chunk_x + world->render_distance; chunk_x++) {
                    // Check if ray point falls in chunk
                    float chunk_min_x = chunk_x * CHUNK_SIZE - 0.5;
                    float chunk_max_x = chunk_x * CHUNK_SIZE + CHUNK_SIZE + 0.5;
                    float chunk_min_y = chunk_y * CHUNK_SIZE - 0.5;
                    float chunk_max_y = chunk_y * CHUNK_SIZE + CHUNK_SIZE + 0.5;
                    float chunk_min_z = chunk_z * CHUNK_SIZE - 0.5;
                    float chunk_max_z = chunk_z * CHUNK_SIZE + CHUNK_SIZE + 0.5;
                    if (
                        ray_point.x >= chunk_min_x && ray_point.x <= chunk_max_x &&
                        ray_point.y >= chunk_min_y && ray_point.y <= chunk_max_y &&
                        ray_point.z >= chunk_min_z && ray_point.z <= chunk_max_z
                    ) {
                        // Get lighted chunk data
                        Chunk* chunk = world_request_chunk(world, chunk_x, chunk_y, chunk_z);
                        if (chunk != NULL) {
                            if (!chunk->is_lighted) {
                                world_request_chunk_update(world, chunk);
                            } else {
                                // Ray point is in chunk check all blocks for collision
                                for (int block_z = 0; block_z < CHUNK_SIZE; block_z++) {
                                    for (int block_y = 0; block_y < CHUNK_SIZE; block_y++) {
                                        for (int block_x = 0; block_x < CHUNK_SIZE; block_x++) {
                                            BlockType block_type = chunk->data[block_z * CHUNK_SIZE * CHUNK_SIZE + block_y * CHUNK_SIZE + block_x];
                                            if ((block_type & CHUNK_DATA_VISIBLE_BIT) != 0) {
                                                // Check if ray point falls in block
                                                float block_min_x = chunk->x * CHUNK_SIZE + block_x - 0.5;
                                                float block_max_x = chunk->x * CHUNK_SIZE + block_x + 0.5;
                                                float block_min_y = chunk->y * CHUNK_SIZE + block_y - 0.5;
                                                float block_max_y = chunk->y * CHUNK_SIZE + block_y + 0.5;
                                                float block_min_z = chunk->z * CHUNK_SIZE + block_z - 0.5;
                                                float block_max_z = chunk->z * CHUNK_SIZE + block_z + 0.5;
                                                if (
                                                    ray_point.x >= block_min_x && ray_point.x <= block_max_x &&
                                                    ray_point.y >= block_min_y && ray_point.y <= block_max_y &&
                                                    ray_point.z >= block_min_z && ray_point.z <= block_max_z
                                                ) {
                                                    // Collision detect return absolute block position
                                                    BlockPosition *block_position = malloc(sizeof(BlockPosition));
                                                    block_position->chunk_x = chunk_x;
                                                    block_position->chunk_y = chunk_y;
                                                    block_position->chunk_z = chunk_z;
                                                    block_position->block_x = block_x;
                                                    block_position->block_y = block_y;
                                                    block_position->block_z = block_z;

                                                    // Check good block side
                                                    float distance = 1;

                                                    if (fabs(ray_point.x - block_min_x) < distance) {
                                                        distance = fabs(ray_point.x - block_min_x);
                                                        block_position->block_side = BLOCK_SIDE_LEFT;
                                                    }
                                                    if (fabs(ray_point.x - block_max_x) < distance) {
                                                        distance = fabs(ray_point.x - block_max_x);
                                                        block_position->block_side = BLOCK_SIDE_RIGHT;
                                                    }

                                                    if (fabs(ray_point.y - block_min_y) < distance) {
                                                        distance = fabs(ray_point.y - block_min_y);
                                                        block_position->block_side = BLOCK_SIDE_BELOW;
                                                    }
                                                    if (fabs(ray_point.y - block_max_y) < distance) {
                                                        distance = fabs(ray_point.y - block_max_y);
                                                        block_position->block_side = BLOCK_SIDE_ABOVE;
                                                    }

                                                    if (fabs(ray_point.z - block_min_z) < distance) {
                                                        distance = fabs(ray_point.z - block_min_z);
                                                        block_position->block_side = BLOCK_SIDE_FRONT;
                                                    }
                                                    if (fabs(ray_point.z - block_max_z) < distance) {
                                                        distance = fabs(ray_point.z - block_max_z);
                                                        block_position->block_side = BLOCK_SIDE_BACK;
                                                    }

                                                    return block_position;
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
        }

        // Increase point by a block and check again
        float increase = 0.1;
        distance += increase;
        Vector4 update = { 0, 0, increase, 1 };
        vector4_mul(&update, &rotation_x_matrix);
        vector4_mul(&update, &rotation_y_matrix);
        vector4_add(&ray_point, &update);
    }

    return NULL;
}

BlockType world_get_block(World* world, BlockPosition* block_position) {
    Chunk* chunk = world_get_chunk(world, block_position->chunk_x, block_position->chunk_y, block_position->chunk_z);
    BlockType block_type = chunk->data[block_position->block_z * CHUNK_SIZE * CHUNK_SIZE + block_position->block_y * CHUNK_SIZE + block_position->block_x];
    block_type &= ~CHUNK_DATA_VISIBLE_BIT;
    return block_type;
}

void world_set_block(World* world, BlockPosition* block_position, BlockType block_type) {
    Chunk* chunk = world_get_chunk(world, block_position->chunk_x, block_position->chunk_y, block_position->chunk_z);
    chunk->is_changed = true;
    chunk->data[block_position->block_z * CHUNK_SIZE * CHUNK_SIZE + block_position->block_y * CHUNK_SIZE + block_position->block_x] = block_type;
    world_request_chunk_update(world, chunk);

    if (block_position->block_x == 0) {
        Chunk* other_chunk =  world_get_chunk(world, block_position->chunk_x - 1, block_position->chunk_y, block_position->chunk_z);
        other_chunk->is_relighted = false;
        world_request_chunk_update(world, other_chunk);
    }

    if (block_position->block_x == CHUNK_SIZE - 1) {
        Chunk* other_chunk =  world_get_chunk(world, block_position->chunk_x + 1, block_position->chunk_y, block_position->chunk_z);
        other_chunk->is_relighted = false;
        world_request_chunk_update(world, other_chunk);
    }

    if (block_position->block_y == 0) {
        Chunk* other_chunk =  world_get_chunk(world, block_position->chunk_x, block_position->chunk_y - 1, block_position->chunk_z);
        other_chunk->is_relighted = false;
        world_request_chunk_update(world, other_chunk);
    }

    if (block_position->block_y == CHUNK_SIZE - 1) {
        Chunk* other_chunk =  world_get_chunk(world, block_position->chunk_x, block_position->chunk_y + 1, block_position->chunk_z);
        other_chunk->is_relighted = false;
        world_request_chunk_update(world, other_chunk);
    }

    if (block_position->block_z == 0) {
        Chunk* other_chunk =  world_get_chunk(world, block_position->chunk_x, block_position->chunk_y, block_position->chunk_z - 1);
        other_chunk->is_relighted = false;
        world_request_chunk_update(world, other_chunk);
    }

    if (block_position->block_z == CHUNK_SIZE - 1) {
        Chunk* other_chunk =  world_get_chunk(world, block_position->chunk_x, block_position->chunk_y, block_position->chunk_z + 1);
        other_chunk->is_relighted = false;
        world_request_chunk_update(world, other_chunk);
    }
}

void world_free(World* world, Camera* camera, BlockType *selected_block_type) {
    mtx_lock(&world->worker_running_lock);
    world->worker_running = false;
    mtx_unlock(&world->worker_running_lock);

    for (int i = 0; i < WORLD_WORKER_THREAD_COUNT; i++) {
        thrd_join(world->worker_threads[i], NULL);
    }

    for (int i = 0; i < WORLD_CHUNK_CACHE_COUNT; i++) {
        Chunk* chunk = world->chunk_cache[i];
        if (chunk != NULL) {
            chunk_free(chunk);
        }
    }

    // Free mutex locks
    mtx_destroy(&world->chunk_cache_lock);
    mtx_destroy(&world->request_queue_lock);

    // Save player position
    database_settings_set_float(world->database, "player_x", camera->position.x);
    database_settings_set_float(world->database, "player_y", camera->position.y);
    database_settings_set_float(world->database, "player_z", camera->position.z);

    // Save player rotation
    database_settings_set_float(world->database, "player_pitch", camera->pitch);
    database_settings_set_float(world->database, "player_yaw", camera->yaw);

    // Save player selected block
    database_settings_set_int(world->database, "player_selected_block_type", *selected_block_type);

    database_free(world->database);

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
                Chunk* chunk = request->arguments.chunk_pointer;
                if (chunk->is_changed) {
                    database_chunks_set_chunk(world->database, chunk);
                }
                chunk_update(chunk, world);
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
