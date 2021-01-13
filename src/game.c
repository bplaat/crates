// PlaatCraft - Game

#include "game.h"
#include <stdio.h>
#include <stdlib.h>
#include <time.h>
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

void game_framebuffer_size_callback(GLFWwindow* window, int width, int height) {
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

        if (key == GLFW_KEY_F3) {
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
    if (button == GLFW_MOUSE_BUTTON_LEFT && action == GLFW_PRESS && !game->is_playing) {
        game->camera->is_first_mouse_movement = true;
        glfwSetInputMode(window, GLFW_CURSOR, GLFW_CURSOR_DISABLED);
        game->is_playing = true;
    }
}

void game_cursor_position_callback(GLFWwindow* window, double xpos, double ypos) {
    Game* game = (Game*)glfwGetWindowUserPointer(window);
    if (game->is_playing) {
        camera_cursor_position_callback(game->camera, xpos, ypos);
    }
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
    #if DEBUG
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
    glfwMakeContextCurrent(game->window);
    glfwSwapInterval(1);

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
    game->blocks_texture_atlas = texture_atlas_new("assets/images/blocks.png", 128);
    game->cursor_texture = texture_new_from_file("assets/images/cursor.png", true);

    // Create camera
    game->camera = camera_new(radians(45), (float)game->width / (float)game->height, 0.1, 1000);
    game->camera->position.x = CHUNK_SIZE / 2;
    game->camera->position.y = 1.5 + CHUNK_SIZE;
    game->camera->position.z = CHUNK_SIZE / 2;
    camera_update_matrix(game->camera);

    // Create world seed
    int64_t seed = time(NULL);
    srand(seed);
    if (rand() % 2 == 0) {
        seed = -seed;
    }

    // Create world
    game->world = world_new(seed);

    // Set callbacks
    glfwSetFramebufferSizeCallback(game->window, game_framebuffer_size_callback);
    glfwSetKeyCallback(game->window, game_key_callback);
    glfwSetMouseButtonCallback(game->window, game_mouse_button_callback);
    glfwSetCursorPosCallback(game->window, game_cursor_position_callback);

    return game;
}

void game_update(Game* game, float delta) {
    // Update camera
    camera_update(game->camera, delta);
}

void game_render(Game* game) {
    glViewport(0, 0, game->width, game->height);

    // Clear screen
    glEnable(GL_DEPTH_TEST);

    glEnable(GL_CULL_FACE);
    glFrontFace(GL_CW);

    glClearColor(176.f / 255.f, 233.f / 255.f, 252.f / 255.f, 1.f);
    glClear(GL_COLOR_BUFFER_BIT | GL_DEPTH_BUFFER_BIT);

    // Render world
    int rendered_chunks = world_render(game->world, game->camera, game->block_shader, game->blocks_texture_atlas);

    // Enable flat shader
    glDisable(GL_DEPTH_TEST);
    flat_shader_enable(game->flat_shader);

    Matrix4 projectionMatrix;
    matrix4_flat_projection(&projectionMatrix, game->width, game->height);
    glUniformMatrix4fv(game->flat_shader->projection_matrix_uniform, 1, GL_FALSE, &projectionMatrix.m11);

    Matrix4 modelMatrix;

    // Render game hude
    if (game->is_playing) {
        // Render cursor
        {
            matrix4_flat_rect(&modelMatrix, (game->width - game->cursor_texture->width) / 2, (game->height - game->cursor_texture->height) / 2, game->cursor_texture->width, game->cursor_texture->height);
            glUniformMatrix4fv(game->flat_shader->model_matrix_uniform, 1, GL_FALSE, &modelMatrix.m11);

            glUniform1i(game->flat_shader->is_textured_uniform, true);
            texture_enable(game->cursor_texture);
            glDrawArrays(GL_TRIANGLES, 0, PLANE_VERTICES_COUNT);
            texture_disable(game->cursor_texture);
        }

        // Debug label
        if (game->is_debugged) {
            // Generate debug label
            #define DEBUG_LINES_COUNT 3
            char debug_lines[DEBUG_LINES_COUNT][128];

            sprintf(
                debug_lines[0],
                "OpenGL %d.%d Core Profile - Render distance: %d chunks - Rendered: %d chunks - Fps: %d",
                GLVersion.major, GLVersion.minor, game->world->render_distance, rendered_chunks, game->fps
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
                "Seed: %ld - Wireframed: %s - Flat shaded: %s",
                game->world->seed,
                game->world->is_wireframed ? "true" : "false",
                game->world->is_flat_shaded ? "true" : "false"
            );

            // Render debug label
            Color text_color = { 17, 17, 17, 255 };
            glUniform1i(game->flat_shader->is_textured_uniform, true);
            for (int i = 0; i < DEBUG_LINES_COUNT; i++) {
                TextTexture* debug_text_texture = text_texture_new(debug_lines[i], game->text_font, 24, text_color);

                matrix4_flat_rect(&modelMatrix, 16, 16 + i * (24 + 16), debug_text_texture->texture->width, debug_text_texture->texture->height);
                glUniformMatrix4fv(game->flat_shader->model_matrix_uniform, 1, GL_FALSE, &modelMatrix.m11);

                texture_enable(debug_text_texture->texture);
                glDrawArrays(GL_TRIANGLES, 0, PLANE_VERTICES_COUNT);
                texture_disable(debug_text_texture->texture);

                text_texture_free(debug_text_texture);
            }
        }
    }

    // Render pause menu
    else {
        // Render darker background overlay
        {
            matrix4_flat_rect(&modelMatrix, 0, 0, game->width + 1, game->height + 1);
            glUniformMatrix4fv(game->flat_shader->model_matrix_uniform, 1, GL_FALSE, &modelMatrix.m11);

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

        #define CONTROLS_LINES_COUNT 8
        int y = (game->height - (64 + 24 + 32 + 32 + (24 + 16) * CONTROLS_LINES_COUNT + 8 + 32)) / 2;

        // Render title text
        {
            TextTexture* title_text_texture = text_texture_new(game->title, game->text_font, 64, text_color);

            matrix4_flat_rect(&modelMatrix, (game->width - title_text_texture->texture->width) / 2, y, title_text_texture->texture->width, title_text_texture->texture->height);
            glUniformMatrix4fv(game->flat_shader->model_matrix_uniform, 1, GL_FALSE, &modelMatrix.m11);
            y += 64 + 24;

            texture_enable(title_text_texture->texture);
            glDrawArrays(GL_TRIANGLES, 0, PLANE_VERTICES_COUNT);
            texture_disable(title_text_texture->texture);
        }

        // Render description text
        {
            TextTexture* description_text_texture = text_texture_new("A modern OpenGL Minecraft like game for a school project about Computer Graphics", game->text_font, 32, text_color);

            matrix4_flat_rect(&modelMatrix, (game->width - description_text_texture->texture->width) / 2, y, description_text_texture->texture->width, description_text_texture->texture->height);
            glUniformMatrix4fv(game->flat_shader->model_matrix_uniform, 1, GL_FALSE, &modelMatrix.m11);
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
                "Use F11 to toggle fullscreen mode",
                "Use Ctrl+R to toggle the render distance (near or far)",
                "Use F3 to toggle debug mode",
                "Use Ctrl+I to toggle wireframe mode",
                "Use Ctrl+T to toggle flat shading mode",
            };

            for (int i = 0; i < CONTROLS_LINES_COUNT; i++) {
                TextTexture* control_text_texture = text_texture_new(controls_lines[i], game->text_font, 24, secondary_text_color);

                matrix4_flat_rect(&modelMatrix, (game->width - control_text_texture->texture->width) / 2, y, control_text_texture->texture->width, control_text_texture->texture->height);
                glUniformMatrix4fv(game->flat_shader->model_matrix_uniform, 1, GL_FALSE, &modelMatrix.m11);
                y += 24 + 16;

                texture_enable(control_text_texture->texture);
                glDrawArrays(GL_TRIANGLES, 0, PLANE_VERTICES_COUNT);
                texture_disable(control_text_texture->texture);

                text_texture_free(control_text_texture);
            }
        }

        // Render footer text
        {
            TextTexture* footer_text_texture = text_texture_new("Made by Bastiaan van der Plaat", game->text_font, 32, text_color);

            y += 8;
            matrix4_flat_rect(&modelMatrix, (game->width - footer_text_texture->texture->width) / 2, y, footer_text_texture->texture->width, footer_text_texture->texture->height);
            glUniformMatrix4fv(game->flat_shader->model_matrix_uniform, 1, GL_FALSE, &modelMatrix.m11);

            texture_enable(footer_text_texture->texture);
            glDrawArrays(GL_TRIANGLES, 0, PLANE_VERTICES_COUNT);
            texture_disable(footer_text_texture->texture);

            text_texture_free(footer_text_texture);
        }
    }

    flat_shader_disable(game->flat_shader);
}

void game_start(Game* game) {
    double fpsTime = 0;
    int frame = 0;
    double oldTime = 0;
    while (!glfwWindowShouldClose(game->window)) {
        glfwPollEvents();

        // Update fps counter
        double time = glfwGetTime();
        if (time - fpsTime >= 1.f) {
            fpsTime = time;
            game->fps = frame;
            frame = 0;
        }

        // Update game
        game_update(game, time - oldTime);
        oldTime = time;

        // Render game
        game_render(game);

        // Swap frame buffers
        glfwSwapBuffers(game->window);

        // Increase frame counter
        frame++;
    }
}

void game_free(Game* game) {
    // Free world
    world_free(game->world);

    // Free camera
    camera_free(game->camera);

    // Free textures
    texture_atlas_free(game->blocks_texture_atlas);
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
