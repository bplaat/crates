// PlaatCraft - Game Header

#ifndef GAME_H
#define GAME_H

#include <stdbool.h>
#define GLFW_INCLUDE_NONE
#include <GLFW/glfw3.h>
#include "camera.h"

typedef enum BlockTexture {
    BLOCK_TEXTURE_GRASS_TOP,
    BLOCK_TEXTURE_GRASS_SIDE,
    BLOCK_TEXTURE_DIRT,
    BLOCK_TEXTURE_WATER,
    BLOCK_TEXTURE_SAND,
    BLOCK_TEXTURE_STONE,
    BLOCK_TEXTURE_COAL,
    BLOCK_TEXTURE_GOLD,
    BLOCK_TEXTURE_TRUNK_TOP,
    BLOCK_TEXTURE_TRUNK_SIDE,
    BLOCK_TEXTURE_LEAVES,
    BLOCK_TEXTURE_CACTUS_TOP,
    BLOCK_TEXTURE_CACTUS_SIDE,
    BLOCK_TEXTURE_SIZE
} BlockTexture;

typedef enum BlockType {
    BLOCK_TYPE_AIR,
    BLOCK_TYPE_GRASS,
    BLOCK_TYPE_DIRT,
    BLOCK_TYPE_WATER,
    BLOCK_TYPE_SAND,
    BLOCK_TYPE_STONE,
    BLOCK_TYPE_COAL,
    BLOCK_TYPE_GOLD,
    BLOCK_TYPE_TRUNK,
    BLOCK_TYPE_LEAVES,
    BLOCK_TYPE_CACTUS,
    BLOCK_TYPE_SIZE
} BlockType;

// BlockTexture BLOCK_FACES[BLOCK_TYPE_SIZE][6];

#define CHUNK_SIZE 16

typedef struct Chunk {
    int x;
    int y;
    int z;
    uint8_t *data;
    uint8_t *visible;
} Chunk;

typedef struct Game {
    char* title;
    int width;
    int height;
    int oldWidth;
    int oldHeight;

    GLFWwindow* window;

    Camera* camera;
    Vector4 velocity;

    bool isFullscreen;
    bool isWireframed;
    bool isCursorLocked;

    bool isMovingForward;
    bool isMovingBackward;
    bool isMovingLeft;
    bool isMovingRight;
    bool isMovingUp;
    bool isMovingDown;

    int render_distance;
    Chunk* chunks[512];
    int chunks_size;
} Game;

#define GAME_MIN_WIDTH 320
#define GAME_MIN_HEIGHT 240

void game_init();

void game_framebuffer_size_callback(GLFWwindow* window, int width, int height);

void game_key_callback(GLFWwindow* window, int key, int scancode, int action, int mods);

void game_mouse_button_callback(GLFWwindow* window, int button, int action, int mods);

void game_cursor_position_callback(GLFWwindow* window, double xpos, double ypos);

Game* game_new(char* title, int width, int height);

void game_update(Game* game, float delta);

void game_render(Game* game);

void game_start(Game* game);

void game_free(Game* game);

#endif
