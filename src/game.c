// PlaatCraft - Game

#include "game.h"
#include <stdio.h>
#include <stdlib.h>
#include "log.h"
#include "math/matrix4.h"
#include "textures/text_texture.h"
#include <emmintrin.h>

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
    (void)scancode;
    (void)mods;
    Game* game = (Game*)glfwGetWindowUserPointer(window);

    if (action == GLFW_PRESS) {
        if (key == GLFW_KEY_W || key == GLFW_KEY_UP) {
            game->isMovingForward = true;
        }
        if (key == GLFW_KEY_S || key == GLFW_KEY_DOWN) {
            game->isMovingBackward = true;
        }
        if (key == GLFW_KEY_A || key == GLFW_KEY_LEFT) {
            game->isMovingLeft = true;
        }
        if (key == GLFW_KEY_D || key == GLFW_KEY_RIGHT) {
            game->isMovingRight = true;
        }
        if (key == GLFW_KEY_SPACE) {
            game->isMovingUp = true;
        }
        if (key == GLFW_KEY_LEFT_SHIFT) {
            game->isMovingDown = true;
        }
    }

    if (action == GLFW_RELEASE) {
        if (key == GLFW_KEY_F11) {
            GLFWmonitor* primary_monitor = glfwGetPrimaryMonitor();
            const GLFWvidmode* primary_screen = glfwGetVideoMode(primary_monitor);
            if (!game->is_fullscreened) {
                game->is_fullscreened = true;
                game->oldWidth = game->width;
                game->oldHeight = game->height;
                glfwSetWindowMonitor(window, primary_monitor, 0, 0, primary_screen->width, primary_screen->height, GLFW_DONT_CARE);
            } else {
                game->is_fullscreened = false;
                glfwSetWindowMonitor(window, NULL, (primary_screen->width - game->oldWidth) / 2, (primary_screen->height - game->oldHeight) / 2, game->oldWidth, game->oldHeight, GLFW_DONT_CARE);
                glfwRestoreWindow(window);
            }
        }

        if (key == GLFW_KEY_ESCAPE && game->is_cursor_locked) {
            game->is_cursor_locked = false;
            glfwSetInputMode(window, GLFW_CURSOR, GLFW_CURSOR_NORMAL);
        }

        if ((mods & GLFW_MOD_CONTROL) != 0 && key == GLFW_KEY_I) {
            game->is_wireframed = !game->is_wireframed;
        }

        if (key == GLFW_KEY_W || key == GLFW_KEY_UP) {
            game->isMovingForward = false;
        }
        if (key == GLFW_KEY_S || key == GLFW_KEY_DOWN) {
            game->isMovingBackward = false;
        }
        if (key == GLFW_KEY_A || key == GLFW_KEY_LEFT) {
            game->isMovingLeft = false;
        }
        if (key == GLFW_KEY_D || key == GLFW_KEY_RIGHT) {
            game->isMovingRight = false;
        }
        if (key == GLFW_KEY_SPACE) {
            game->isMovingUp = false;
        }
        if (key == GLFW_KEY_LEFT_SHIFT) {
            game->isMovingDown = false;
        }
    }
}

// Temp / working on variables
double lastX, lastY;
float yaw;
float pitch;

void game_mouse_button_callback(GLFWwindow* window, int button, int action, int mods) {
    (void)action;
    (void)mods;
    Game* game = (Game*)glfwGetWindowUserPointer(window);

    if (button == GLFW_MOUSE_BUTTON_LEFT && action == GLFW_PRESS && !game->is_cursor_locked) {
        glfwGetCursorPos(window, &lastX, &lastY);
        glfwSetInputMode(window, GLFW_CURSOR, GLFW_CURSOR_DISABLED);
        game->is_cursor_locked = true;
    }
}

void game_cursor_position_callback(GLFWwindow* window, double xpos, double ypos) {
    Game* game = (Game*)glfwGetWindowUserPointer(window);
    if (game->is_cursor_locked) {
        float xoffset = xpos - lastX;
        float yoffset = lastY - ypos;
        lastX = xpos;
        lastY = ypos;

        float sensitivity = 0.002;
        xoffset *= sensitivity;
        yoffset *= sensitivity;

        yaw -= xoffset;
        pitch += yoffset;

        if (pitch > radians(90)) pitch = radians(90);
        if (pitch < -radians(90)) pitch = -radians(90);

        game->camera->rotation.x = -pitch;
        game->camera->rotation.y = -yaw;
    }
}

Game* game_new(char *title, int width, int height) {
    Game* game = malloc(sizeof(Game));
    game->title = title;
    game->width = width;
    game->height = height;

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

    GLFWmonitor* primaryMonitor = glfwGetPrimaryMonitor();
    const GLFWvidmode* primaryScreen = glfwGetVideoMode(primaryMonitor);
    glfwSetWindowPos(game->window, (primaryScreen->width - width) / 2, (primaryScreen->height - height) / 2);

    glfwMakeContextCurrent(game->window);
    glfwSwapInterval(1);

    game->fps = 0;

    glfwSetWindowUserPointer(game->window, game);
    glfwSetFramebufferSizeCallback(game->window, game_framebuffer_size_callback);
    glfwSetKeyCallback(game->window, game_key_callback);
    glfwSetMouseButtonCallback(game->window, game_mouse_button_callback);
    glfwSetCursorPosCallback(game->window, game_cursor_position_callback);

    if (!gladLoadGLLoader((GLADloadproc)glfwGetProcAddress)) {
        log_error("Glad can't load OpenGL context");
    }
    log_info("Using an OpenGL %d.%d context", GLVersion.major, GLVersion.minor);

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
    // game->camera->position.x = CHUNK_SIZE / 2;
    // game->camera->position.y = 1.5 + CHUNK_SIZE;
    // game->camera->position.z = CHUNK_SIZE / 2;
    // camera_update_matrix(game->camera);

    game->velocity.x = 0;
    game->velocity.y = 0;
    game->velocity.z = 0;

    game->isMovingForward = false;
    game->isMovingLeft = false;
    game->isMovingRight = false;
    game->isMovingBackward = false;
    game->isMovingUp = false;
    game->isMovingDown = false;

    // Create world
    game->world = world_new(0);

    return game;
}

void game_update(Game *game, float delta) {
    // Update player camera
    game->velocity.z -= game->velocity.z * 10 * delta;
    game->velocity.x -= game->velocity.x * 10 * delta;
    game->velocity.y -= game->velocity.y * 10 * delta;

    float speed = 5 * delta;

    if (game->isMovingForward) {
        game->velocity.z += speed;
    }
    if (game->isMovingBackward) {
        game->velocity.z -= speed;
    }

    if (game->isMovingLeft) {
        game->velocity.x -=speed;
    }
    if (game->isMovingRight) {
        game->velocity.x += speed;
    }

    if (game->isMovingUp) {
        game->velocity.y += speed;
    }
    if (game->isMovingDown) {
        game->velocity.y -= speed;
    }

    Matrix4 rotation_matrix;
    matrix4_rotate_x(&rotation_matrix, game->camera->rotation.x);

    Matrix4 temp_matrix;
    matrix4_rotate_y(&temp_matrix, game->camera->rotation.y);
    matrix4_mul(&rotation_matrix, &temp_matrix);

    Vector4 update = game->velocity;
    vector4_mul(&update, &rotation_matrix);

    if (game->isMovingUp || game->isMovingDown) {
        update.x = 0;
        update.z = 0;
        update.y = game->velocity.y;
    } else {
        update.y = 0;
    }

    vector4_add(&game->camera->position, &update);

    camera_update_matrix(game->camera);
}

void game_render(Game *game) {
    glViewport(0, 0, game->width, game->height);

    // Clear screen
    glEnable(GL_DEPTH_TEST);
    glClearColor(176.f / 255.f, 233.f / 255.f, 252.f / 255.f, 1.f);
    glClear(GL_COLOR_BUFFER_BIT | GL_DEPTH_BUFFER_BIT);

    // Render world
    int render_distance = 4;
    glPolygonMode(GL_FRONT_AND_BACK, game->is_wireframed ? GL_LINE : GL_FILL);
    world_render(game->world, game->camera, game->block_shader, game->blocks_texture_atlas, render_distance);
    glPolygonMode(GL_FRONT_AND_BACK, GL_FILL);

    // Enable flat shader
    flat_shader_enable(game->flat_shader);

    Matrix4 projectionMatrix;
    matrix4_flat_projection(&projectionMatrix, game->width, game->height);
    glUniformMatrix4fv(game->flat_shader->projection_matrix_uniform, 1, GL_FALSE, &projectionMatrix.m11);

    // Render cursor
    Matrix4 modelMatrix;
    matrix4_flat_rect(&modelMatrix, (game->width - game->cursor_texture->width) / 2, (game->height - game->cursor_texture->height) / 2, game->cursor_texture->width, game->cursor_texture->height);
    glUniformMatrix4fv(game->flat_shader->model_matrix_uniform, 1, GL_FALSE, &modelMatrix.m11);

    texture_enable(game->cursor_texture);
    glDrawArrays(GL_TRIANGLES, 0, 6);
    texture_disable(game->cursor_texture);

    // Debug label
    #if DEBUG
        // Generate debug label
        Color text_color = { 17, 17, 17, 255 };

        int player_chunk_x = floor(game->camera->position.x / (float)CHUNK_SIZE);
        int player_chunk_y = floor(game->camera->position.y / (float)CHUNK_SIZE);
        int player_chunk_z = floor(game->camera->position.z / (float)CHUNK_SIZE);

        char debug_text[256];
        sprintf(
            debug_text,
            "Fps: %d - Position: %.02f %.02f %.02f - Rotation: %.03f %.03f - Chunk: %d %d %d - Render distance: %d chunks",
            game->fps,
            game->camera->position.x, game->camera->position.y, game->camera->position.z,
            game->camera->rotation.x, game->camera->rotation.y,
            player_chunk_x, player_chunk_y, player_chunk_z,
            render_distance
        );

        // Render debug label
        TextTexture* debug_text_texture = text_texture_new(debug_text, game->text_font, 24, text_color);

        matrix4_flat_rect(&modelMatrix, 16, 16, debug_text_texture->texture->width, debug_text_texture->texture->height);
        glUniformMatrix4fv(game->flat_shader->model_matrix_uniform, 1, GL_FALSE, &modelMatrix.m11);

        texture_enable(debug_text_texture->texture);
        glDrawArrays(GL_TRIANGLES, 0, 6);
        texture_disable(debug_text_texture->texture);

        text_texture_free(debug_text_texture);
    #endif

    flat_shader_disable(game->flat_shader);
}

void game_start(Game *game) {
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
