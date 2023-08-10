// PlaatCraft - Game

#include "game.h"
#include <stdio.h>
#include <stdlib.h>
#include <inttypes.h>
#include "config.h"
#include "log.h"
#include "math/matrix4.h"
#include "textures/text_texture.h"

void game_init(void) {
    if (!glfwInit()) {
        log_error("Can't init GLFW");
    }
    atexit(glfwTerminate);
}

void game_size_callback(GLFWwindow* window, int width, int height) {
    Game* game = (Game*)glfwGetWindowUserPointer(window);
    game->width = width;
    game->height = height;

    game->camera->aspect = (float)width / (float)height;
}

void game_key_callback(GLFWwindow* window, int key, int scancode, int action, int mods) {
    Game* game = (Game*)glfwGetWindowUserPointer(window);

    camera_key_callback(game->camera, key, scancode, action, mods);

    if (action == GLFW_RELEASE) {
        if (key == GLFW_KEY_F11) {
            GLFWmonitor* primary_monitor = glfwGetPrimaryMonitor();
            const GLFWvidmode* primary_screen = glfwGetVideoMode(primary_monitor);
            if (!game->is_fullscreen) {
                game->is_fullscreen = true;
                game->oldWidth = game->width;
                game->oldHeight = game->height;
                glfwSetWindowMonitor(window, primary_monitor, 0, 0, primary_screen->width, primary_screen->height, GLFW_DONT_CARE);
            } else {
                game->is_fullscreen = false;
                glfwSetWindowMonitor(window, NULL, (primary_screen->width - game->oldWidth) / 2, (primary_screen->height - game->oldHeight) / 2, game->oldWidth, game->oldHeight, GLFW_DONT_CARE);
                glfwRestoreWindow(window);
            }
        }

        if (key == GLFW_KEY_ESCAPE && game->is_playing) {
            game->is_playing = false;
            glfwSetInputMode(window, GLFW_CURSOR, GLFW_CURSOR_NORMAL);
        }

        if (key == GLFW_KEY_F3 || ((mods & GLFW_MOD_CONTROL) != 0 && key == GLFW_KEY_Y)) {
            game->is_debugged = !game->is_debugged;
        }

        if ((mods & GLFW_MOD_CONTROL) != 0 && key == GLFW_KEY_T) {
            game->world->is_flat_shaded = !game->world->is_flat_shaded;
        }

        if ((mods & GLFW_MOD_CONTROL) != 0 && key == GLFW_KEY_I) {
            game->world->is_wireframed = !game->world->is_wireframed;
        }

        if ((mods & GLFW_MOD_CONTROL) != 0 && key == GLFW_KEY_R) {
            if (game->world->render_distance == WORLD_RENDER_DISTANCE_NEAR) {
                game->world->render_distance = WORLD_RENDER_DISTANCE_FAR;
            } else {
                game->world->render_distance = WORLD_RENDER_DISTANCE_NEAR;
            }
        }
    }
}

void game_mouse_button_callback(GLFWwindow* window, int button, int action, int mods) {
    (void)mods;
    Game* game = (Game*)glfwGetWindowUserPointer(window);
    if (game->is_playing) {
        if (game->selected_block != NULL) {
            if (button == GLFW_MOUSE_BUTTON_LEFT && action == GLFW_RELEASE) {
                world_set_block(game->world, game->selected_block, BLOCK_TYPE_AIR);
            }

            if (button == GLFW_MOUSE_BUTTON_MIDDLE && action == GLFW_RELEASE) {
                game->selected_block_type = world_get_block(game->world, game->selected_block);
            }

            if (button == GLFW_MOUSE_BUTTON_RIGHT && action == GLFW_RELEASE) {
                BlockPosition block_position = *game->selected_block;

                if (block_position.block_side == BLOCK_SIDE_LEFT) {
                    if (block_position.block_x == 0) {
                        block_position.chunk_x--;
                        block_position.block_x = CHUNK_SIZE - 1;
                    } else {
                        block_position.block_x--;
                    }
                }
                if (block_position.block_side == BLOCK_SIDE_RIGHT) {
                    if (block_position.block_x == CHUNK_SIZE - 1) {
                        block_position.chunk_x++;
                        block_position.block_x = 0;
                    } else {
                        block_position.block_x++;
                    }
                }

                if (block_position.block_side == BLOCK_SIDE_ABOVE) {
                    if (block_position.block_y == CHUNK_SIZE - 1) {
                        block_position.chunk_y++;
                        block_position.block_y = 0;
                    } else {
                        block_position.block_y++;
                    }
                }
                if (block_position.block_side == BLOCK_SIDE_BELOW) {
                    if (block_position.block_y == 0) {
                        block_position.chunk_y--;
                        block_position.block_y = CHUNK_SIZE - 1;
                    } else {
                        block_position.block_y--;
                    }
                }

                if (block_position.block_side == BLOCK_SIDE_FRONT) {
                    if (block_position.block_z == 0) {
                        block_position.chunk_z--;
                        block_position.block_z = CHUNK_SIZE - 1;
                    } else {
                        block_position.block_z--;
                    }
                }
                if (block_position.block_side == BLOCK_SIDE_BACK) {
                    if (block_position.block_z == CHUNK_SIZE - 1) {
                        block_position.chunk_z++;
                        block_position.block_z = 0;
                    } else {
                        block_position.block_z++;
                    }
                }

                world_set_block(game->world, &block_position, game->selected_block_type);
            }
        }
    } else {
        if (button == GLFW_MOUSE_BUTTON_LEFT && action == GLFW_RELEASE) {
            game->camera->is_first_mouse_movement = true;
            glfwSetInputMode(window, GLFW_CURSOR, GLFW_CURSOR_DISABLED);
            game->is_playing = true;
        }
    }
}

void game_cursor_position_callback(GLFWwindow* window, double xpos, double ypos) {
    Game* game = (Game*)glfwGetWindowUserPointer(window);
    if (game->is_playing) {
        camera_cursor_position_callback(game->camera, xpos, ypos);
    }
}

void game_scroll_callback(GLFWwindow* window, double xoffset, double yoffset) {
    (void)xoffset;
    Game* game = (Game*)glfwGetWindowUserPointer(window);
    if (game->is_playing) {
        if (yoffset < 0) {
            if (game->selected_block_type == BLOCK_TYPE_SIZE - 2) { // Skip Selected block type
                game->selected_block_type = 1; // Skip Air block type
            } else {
                game->selected_block_type++;
            }
        }
        if (yoffset > 0) {
            if (game->selected_block_type == 1) { // Skip Air block type
                game->selected_block_type = BLOCK_TYPE_SIZE - 2; // Skip Selected block type
            } else {
                game->selected_block_type--;
            }
        }
    }
}

void game_content_scale_callback(GLFWwindow* window, float scale_x, float scale_y) {
    Game* game = (Game*)glfwGetWindowUserPointer(window);
    game->scale_x = scale_x;
    game->scale_y = scale_y;
}

Game* game_new(char* title, int width, int height) {
    Game* game = malloc(sizeof(Game));
    game->title = title;
    game->width = width;
    game->height = height;

    // Create window
    glfwWindowHint(GLFW_CONTEXT_VERSION_MAJOR, 3);
    glfwWindowHint(GLFW_CONTEXT_VERSION_MINOR, 3);
    glfwWindowHint(GLFW_OPENGL_PROFILE, GLFW_OPENGL_CORE_PROFILE);
    #ifdef __APPLE__
        glfwWindowHint(GLFW_OPENGL_FORWARD_COMPAT, GL_TRUE);
    #endif

    game->window = glfwCreateWindow(width, height, title, NULL, NULL);
    if (!game->window) {
        log_error("Can't create glfw window");
    }

    // Init game values
    game->is_playing = false;
    game->is_fullscreen = false;
    #ifdef DEBUG
        game->is_debugged = true;
    #else
        game->is_debugged = false;
    #endif
    game->fps = 0;

    // Center window
    GLFWmonitor* primaryMonitor = glfwGetPrimaryMonitor();
    const GLFWvidmode* primaryScreen = glfwGetVideoMode(primaryMonitor);
    glfwSetWindowPos(game->window, (primaryScreen->width - width) / 2, (primaryScreen->height - height) / 2);

    // Set some window props
    glfwSetWindowUserPointer(game->window, game);
    glfwSetWindowSizeLimits(game->window, width / 2, height / 2, GLFW_DONT_CARE, GLFW_DONT_CARE);
    glfwGetWindowContentScale(game->window, &game->scale_x, &game->scale_y);
    glfwMakeContextCurrent(game->window);

    // Load OpenGL
    if (!gladLoadGLLoader((GLADloadproc)glfwGetProcAddress)) {
        log_error("Glad can't load the OpenGL 3.3 Core Profile");
    }
    log_info("Using the OpenGL %d.%d Core Profile", GLVersion.major, GLVersion.minor);

    // Load fonts
    game->text_font = font_new("assets/fonts/Montserrat-Regular.ttf");

    // Load shaders
    game->block_shader = block_shader_new();
    game->flat_shader = flat_shader_new();

    // Load textures
    game->blocks_texture_atlas = texture_atlas_new("assets/images/blocks.png", 128, false);
    game->selected_texture_atlas = texture_atlas_new("assets/images/selected.png", 128, true);
    game->cursor_texture = texture_new_from_file("assets/images/cursor.png", true);

    // Create camera
    game->camera = camera_new(radians(45), (float)game->width / (float)game->height, 0.1, 1000);

    // Create world (and load world database)
    game->world = world_new(game->camera, &game->selected_block_type);

    // Set selected block
    game->selected_block = NULL;
    game->selected_block_rotation.x = 0;
    game->selected_block_rotation.y = 0;

    // Set callbacks
    glfwSetWindowSizeCallback(game->window, game_size_callback);
    glfwSetKeyCallback(game->window, game_key_callback);
    glfwSetMouseButtonCallback(game->window, game_mouse_button_callback);
    glfwSetCursorPosCallback(game->window, game_cursor_position_callback);
    glfwSetScrollCallback(game->window, game_scroll_callback);
    glfwSetWindowContentScaleCallback(game->window, game_content_scale_callback);

    return game;
}

void game_update(Game* game, float delta) {
    // Update camera
    camera_update(game->camera, delta);

    // Get selected block
    if (game->selected_block != NULL) {
        free(game->selected_block);
    }
    game->selected_block = world_get_selected_block(game->world, game->camera);

    // Update selected block rotation
    game->selected_block_rotation.x += radians(180) * delta;
    game->selected_block_rotation.z += radians(180) * delta;
}

void game_render(Game* game, float delta) {
    // Init OpenGL viewport and culling
    glViewport(0, 0, game->width * game->scale_x, game->height * game->scale_y);
    glEnable(GL_CULL_FACE);
    glFrontFace(GL_CW);

    // Clear screen
    glClearColor(0.69, 0.91, 0.99, 1);
    glClear(GL_COLOR_BUFFER_BIT | GL_DEPTH_BUFFER_BIT);

    // Render world
    glEnable(GL_DEPTH_TEST);
    int rendered_chunks = world_render(game->world, game->camera, game->block_shader, game->blocks_texture_atlas);

    // Render select block outline
    Matrix4 model_matrix;
    if (game->is_playing && game->selected_block != NULL) {
        block_shader_enable(game->block_shader);
        texture_atlas_enable(game->selected_texture_atlas);

        if (game->world->is_wireframed) {
            glPolygonMode(GL_FRONT_AND_BACK, GL_LINE);
        }

        glUniform1i(game->block_shader->is_lighted_uniform, false);
        glUniform1i(game->block_shader->is_flad_shaded_uniform, game->world->is_flat_shaded);

        glUniformMatrix4fv(game->block_shader->projection_matrix_uniform, 1, GL_FALSE, &game->camera->projection_matrix.elements[0]);
        glUniformMatrix4fv(game->block_shader->view_matrix_uniform, 1, GL_FALSE, &game->camera->view_matrix.elements[0]);

        Vector4 translate_vector = {
            game->selected_block->chunk_x * CHUNK_SIZE + game->selected_block->block_x,
            game->selected_block->chunk_y * CHUNK_SIZE + game->selected_block->block_y,
            -(game->selected_block->chunk_z * CHUNK_SIZE + game->selected_block->block_z),
            1
        };
        matrix4_translate(&model_matrix, &translate_vector);

        Matrix4 scale_matrix;
        Vector4 scale_vector = { 1.02, 1.02, 1.02, 1 };
        matrix4_scale(&scale_matrix, &scale_vector);
        matrix4_mul_matrix4(&model_matrix, &scale_matrix);

        glUniformMatrix4fv(game->block_shader->model_matrix_uniform, 1, GL_FALSE, &model_matrix.elements[0]);

        glUniform1iv(game->block_shader->texture_indexes_uniform, 6, (const GLint*)&BLOCK_TYPE_TEXTURE_FACES[BLOCK_TYPE_SELECTED]);

        glDrawArrays(GL_TRIANGLES, 0, BLOCK_VERTICES_COUNT);

        if (game->world->is_wireframed) {
            glPolygonMode(GL_FRONT_AND_BACK, GL_FILL);
        }

        texture_atlas_disable(game->selected_texture_atlas);
        block_shader_disable(game->block_shader);
    }

    glDisable(GL_DEPTH_TEST);

    // Enable flat shader
    flat_shader_enable(game->flat_shader);

    Matrix4 projection_matrix;
    matrix4_flat_projection(&projection_matrix, game->width, game->height);
    glUniformMatrix4fv(game->flat_shader->projection_matrix_uniform, 1, GL_FALSE, &projection_matrix.elements[0]);

    // Render game hude
    if (game->is_playing) {
        // Debug label
        glUniform1i(game->flat_shader->is_textured_uniform, true);
        Color text_color = { 17, 17, 17, 255 };
        if (game->is_debugged) {
            // Generate debug label
            #define DEBUG_LINES_COUNT 4
            char debug_lines[DEBUG_LINES_COUNT][128];

            sprintf(
                debug_lines[0],
                "OpenGL %d.%d Core Profile - Render distance: %d chunks - Rendered: %d / %d chunks - Fps: %d",
                GLVersion.major, GLVersion.minor, game->world->render_distance, rendered_chunks,
                (game->world->render_distance * 2 + 1) * (game->world->render_distance * 2 + 1) * (game->world->render_distance * 2 + 1),
                game->fps
            );

            int player_chunk_x = floor(game->camera->position.x / (float)CHUNK_SIZE);
            int player_chunk_y = floor(game->camera->position.y / (float)CHUNK_SIZE);
            int player_chunk_z = floor(game->camera->position.z / (float)CHUNK_SIZE);
            sprintf(
                debug_lines[1],
                "Chunk: %d %d %d - Position: %.02f %.02f %.02f - Rotation: %.03f %.03f",
                player_chunk_x, player_chunk_y, player_chunk_z,
                game->camera->position.x, game->camera->position.y, game->camera->position.z,
                game->camera->rotation.x, game->camera->rotation.y
            );

            sprintf(
                debug_lines[2],
                "Seed: %"PRId64" - Wireframed: %s - Flat shaded: %s - Delta: %.04f",
                game->world->seed,
                game->world->is_wireframed ? "true" : "false",
                game->world->is_flat_shaded ? "true" : "false",
                delta
            );

            if (game->selected_block != NULL) {
                sprintf(
                    debug_lines[3],
                    "Selected block: %d %d %d chunk, %d %d %d block, %s face",
                    game->selected_block->chunk_x,
                    game->selected_block->chunk_y,
                    game->selected_block->chunk_z,
                    game->selected_block->block_x,
                    game->selected_block->block_y,
                    game->selected_block->block_z,
                    BLOCK_SIDE_NAMES[game->selected_block->block_side]
                );
            } else {
                sprintf(
                    debug_lines[3],
                    "Selected block: none"
                );
            }

            // Render debug label
            for (int i = 0; i < DEBUG_LINES_COUNT; i++) {
                TextTexture* debug_text_texture = text_texture_new(debug_lines[i], game->text_font, 24, text_color);

                matrix4_flat_rect(&model_matrix, 16, 16 + i * (24 + 16), debug_text_texture->texture->width, debug_text_texture->texture->height);
                glUniformMatrix4fv(game->flat_shader->model_matrix_uniform, 1, GL_FALSE, &model_matrix.elements[0]);

                texture_enable(debug_text_texture->texture);
                glDrawArrays(GL_TRIANGLES, 0, PLANE_VERTICES_COUNT);
                texture_disable(debug_text_texture->texture);

                text_texture_free(debug_text_texture);
            }
        }

        // Render cursor
        {
            matrix4_flat_rect(&model_matrix, (game->width - game->cursor_texture->width) / 2, (game->height - game->cursor_texture->height) / 2, game->cursor_texture->width, game->cursor_texture->height);
            glUniformMatrix4fv(game->flat_shader->model_matrix_uniform, 1, GL_FALSE, &model_matrix.elements[0]);

            texture_enable(game->cursor_texture);
            glDrawArrays(GL_TRIANGLES, 0, PLANE_VERTICES_COUNT);
            texture_disable(game->cursor_texture);
        }

        // Render selected block text
        {
            TextTexture* block_text_texture = text_texture_new(BLOCK_TYPE_NAMES[game->selected_block_type], game->text_font, 48, text_color);

            float size = (game->width / 2) * 0.3;
            matrix4_flat_rect(&model_matrix, size * 1.15, (game->height - size) + (size - block_text_texture->texture->height) / 2, block_text_texture->texture->width, block_text_texture->texture->height);
            glUniformMatrix4fv(game->flat_shader->model_matrix_uniform, 1, GL_FALSE, &model_matrix.elements[0]);

            texture_enable(block_text_texture->texture);
            glDrawArrays(GL_TRIANGLES, 0, PLANE_VERTICES_COUNT);
            texture_disable(block_text_texture->texture);

            text_texture_free(block_text_texture);
        }

        // Render selected block
        {
            flat_shader_disable(game->flat_shader);

            block_shader_enable(game->block_shader);
            texture_atlas_enable(game->blocks_texture_atlas);

            if (game->world->is_wireframed) {
                glPolygonMode(GL_FRONT_AND_BACK, GL_LINE);
            }

            glUniform1i(game->block_shader->is_lighted_uniform, false);
            glUniform1i(game->block_shader->is_flad_shaded_uniform, game->world->is_flat_shaded);

            glUniformMatrix4fv(game->block_shader->projection_matrix_uniform, 1, GL_FALSE, &game->camera->projection_matrix.elements[0]);

            Matrix4 view_matrix;
            matrix4_identity(&view_matrix);
            glUniformMatrix4fv(game->block_shader->view_matrix_uniform, 1, GL_FALSE, &view_matrix.elements[0]);

            Vector4 block_position_camera = { -1.2, -0.6, -2, 1 };
            matrix4_translate(&model_matrix, &block_position_camera);

            Matrix4 temp_matrix;
            matrix4_rotate_x(&temp_matrix, game->selected_block_rotation.x + radians(90));
            matrix4_mul_matrix4(&model_matrix, &temp_matrix);

            matrix4_rotate_z(&temp_matrix, game->selected_block_rotation.z);
            matrix4_mul_matrix4(&model_matrix, &temp_matrix);

            Vector4 block_scale_vector = { 0.2, 0.2, 0.2, 1 };
            matrix4_scale(&temp_matrix, &block_scale_vector);
            matrix4_mul_matrix4(&model_matrix, &temp_matrix);

            glUniformMatrix4fv(game->block_shader->model_matrix_uniform, 1, GL_FALSE, &model_matrix.elements[0]);

            glUniform1iv(game->block_shader->texture_indexes_uniform, 6, (const GLint*)&BLOCK_TYPE_TEXTURE_FACES[game->selected_block_type]);

            glDrawArrays(GL_TRIANGLES, 0, BLOCK_VERTICES_COUNT);

            if (game->world->is_wireframed) {
                glPolygonMode(GL_FRONT_AND_BACK, GL_FILL);
            }

            texture_atlas_disable(game->blocks_texture_atlas);
            block_shader_disable(game->block_shader);
        }
    }

    // Render pause menu
    else {
        // Render darker background overlay
        {
            matrix4_flat_rect(&model_matrix, 0, 0, game->width + 1, game->height + 1);
            glUniformMatrix4fv(game->flat_shader->model_matrix_uniform, 1, GL_FALSE, &model_matrix.elements[0]);

            glUniform1i(game->flat_shader->is_textured_uniform, false);
            glUniform4f(game->flat_shader->color_uniform, 0, 0, 0, 0.75);

            glEnable(GL_BLEND);
            glBlendFunc(GL_SRC_ALPHA, GL_ONE_MINUS_SRC_ALPHA);

            glDrawArrays(GL_TRIANGLES, 0, PLANE_VERTICES_COUNT);

            glDisable(GL_BLEND);
        }

        // Setup text lines render
        glUniform1i(game->flat_shader->is_textured_uniform, true);

        Color text_color = { 255, 255, 255, 255 };
        Color secondary_text_color = { 170, 170, 170, 255 };

        #define CONTROLS_LINES_COUNT 12
        int y = (game->height - (64 + 24 + 32 + 32 + (24 + 8) * CONTROLS_LINES_COUNT + 8 + 32)) / 2;

        // Render title text
        {
            TextTexture* title_text_texture = text_texture_new(game->title, game->text_font, 64, text_color);

            matrix4_flat_rect(&model_matrix, (game->width - title_text_texture->texture->width) / 2, y, title_text_texture->texture->width, title_text_texture->texture->height);
            glUniformMatrix4fv(game->flat_shader->model_matrix_uniform, 1, GL_FALSE, &model_matrix.elements[0]);
            y += 64 + 24;

            texture_enable(title_text_texture->texture);
            glDrawArrays(GL_TRIANGLES, 0, PLANE_VERTICES_COUNT);
            texture_disable(title_text_texture->texture);
        }

        // Render description text
        {
            TextTexture* description_text_texture = text_texture_new("A modern OpenGL Minecraft like game for a school project about Computer Graphics", game->text_font, 32, text_color);

            matrix4_flat_rect(&model_matrix, (game->width - description_text_texture->texture->width) / 2, y, description_text_texture->texture->width, description_text_texture->texture->height);
            glUniformMatrix4fv(game->flat_shader->model_matrix_uniform, 1, GL_FALSE, &model_matrix.elements[0]);
            y += 32 + 32;

            texture_enable(description_text_texture->texture);
            glDrawArrays(GL_TRIANGLES, 0, PLANE_VERTICES_COUNT);
            texture_disable(description_text_texture->texture);

            text_texture_free(description_text_texture);
        }

        // Render controls text
        {
            char* controls_lines[CONTROLS_LINES_COUNT] = {
                "Clink to play the game, use ESC to pause",
                "Use the WASD keys to walk around",
                "Use your mouse pointer to look around",
                "Use the left mouse button to break a block",
                "Use the middle mouse button to select a block",
                "Use the right mouse button to place a block",
                "Use the scroll wheel to select other blocks",
                "Use F11 to toggle fullscreen mode",
                "Use Ctrl+R to toggle the render distance (near or far)",
                "Use F3 or Ctrl+Y to toggle debug mode",
                "Use Ctrl+I to toggle wireframe mode",
                "Use Ctrl+T to toggle flat shading mode",
            };

            for (int i = 0; i < CONTROLS_LINES_COUNT; i++) {
                TextTexture* control_text_texture = text_texture_new(controls_lines[i], game->text_font, 24, secondary_text_color);

                matrix4_flat_rect(&model_matrix, (game->width - control_text_texture->texture->width) / 2, y, control_text_texture->texture->width, control_text_texture->texture->height);
                glUniformMatrix4fv(game->flat_shader->model_matrix_uniform, 1, GL_FALSE, &model_matrix.elements[0]);
                y += 24 + 8;

                texture_enable(control_text_texture->texture);
                glDrawArrays(GL_TRIANGLES, 0, PLANE_VERTICES_COUNT);
                texture_disable(control_text_texture->texture);

                text_texture_free(control_text_texture);
            }
        }

        // Render footer text
        {
            TextTexture* footer_text_texture = text_texture_new("Made by Bastiaan van der Plaat", game->text_font, 32, text_color);

            y += 16;
            matrix4_flat_rect(&model_matrix, (game->width - footer_text_texture->texture->width) / 2, y, footer_text_texture->texture->width, footer_text_texture->texture->height);
            glUniformMatrix4fv(game->flat_shader->model_matrix_uniform, 1, GL_FALSE, &model_matrix.elements[0]);

            texture_enable(footer_text_texture->texture);
            glDrawArrays(GL_TRIANGLES, 0, PLANE_VERTICES_COUNT);
            texture_disable(footer_text_texture->texture);

            text_texture_free(footer_text_texture);
        }

        flat_shader_disable(game->flat_shader);
    }
}

void game_start(Game* game) {
    double fpsTime = glfwGetTime();
    int frame = 0;
    double oldTime = glfwGetTime();
    while (!glfwWindowShouldClose(game->window)) {
        glfwPollEvents();

        // Update fps counter
        double time = glfwGetTime();
        if (time - fpsTime >= 1) {
            fpsTime = time;
            game->fps = frame;
            frame = 0;
        }

        // Update game
        float delta = time - oldTime;
        game_update(game, delta);

        // Render game
        game_render(game, delta);

        // Swap frame buffers
        glfwSwapBuffers(game->window);

        // New time is old time
        oldTime = time;

        // Increase frame counter
        frame++;
    }
}

void game_free(Game* game) {
    // Free world
    world_free(game->world, game->camera, &game->selected_block_type);

    // Free camera
    camera_free(game->camera);

    // Free textures
    texture_atlas_free(game->blocks_texture_atlas);
    texture_atlas_free(game->selected_texture_atlas);
    texture_free(game->cursor_texture);

    // Free shaders
    flat_shader_free(game->flat_shader);
    block_shader_free(game->block_shader);

    // Free fonts
    font_free(game->text_font);

    // Free window
    glfwDestroyWindow(game->window);

    // Free game object
    free(game);
}
