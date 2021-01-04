// PlaatCraft - Game Header

#ifndef GAME_H
#define GAME_H

#include <stdbool.h>
#define GLFW_INCLUDE_NONE
#include <GLFW/glfw3.h>
#include "shaders/block_shader.h"
#include "shaders/flat_shader.h"
#include "textures/texture.h"
#include "textures/texture_atlas.h"
#include "camera.h"
#include "world.h"

typedef struct Game {
    char* title;
    int width;
    int height;
    int oldWidth;
    int oldHeight;

    GLFWwindow* window;
    bool isFullscreen;
    bool isWireframed;
    bool isCursorLocked;

    BlockShader* blockShader;
    FlatShader* flatShader;

    TextureAtlas* blocksTextureAtlas;
    Texture* cursorTexture;

    Camera* camera;
    Vector4 velocity;
    bool isMovingForward;
    bool isMovingBackward;
    bool isMovingLeft;
    bool isMovingRight;
    bool isMovingUp;
    bool isMovingDown;

    World* world;
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
