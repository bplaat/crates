// PlaatCraft - Game

#include "game.h"
#include <stdlib.h>
#include <math.h>
#include "glad.h"
#include "log.h"
#include "shader.h"
#include "texture_atlas.h"
#include "matrix4.h"

BlockTexture BLOCK_FACES[BLOCK_TYPE_SIZE][6] = {
    { 0, 0, 0, 0, 0, 0 }, // Air
    { BLOCK_TEXTURE_GRASS_TOP, BLOCK_TEXTURE_GRASS_SIDE, BLOCK_TEXTURE_GRASS_SIDE, BLOCK_TEXTURE_GRASS_SIDE, BLOCK_TEXTURE_GRASS_SIDE, BLOCK_TEXTURE_DIRT }, // Grass
    { BLOCK_TEXTURE_DIRT, BLOCK_TEXTURE_DIRT, BLOCK_TEXTURE_DIRT, BLOCK_TEXTURE_DIRT, BLOCK_TEXTURE_DIRT, BLOCK_TEXTURE_DIRT }, // Dirt
    { BLOCK_TEXTURE_WATER, BLOCK_TEXTURE_WATER, BLOCK_TEXTURE_WATER, BLOCK_TEXTURE_WATER, BLOCK_TEXTURE_WATER, BLOCK_TEXTURE_WATER }, // Water
    { BLOCK_TEXTURE_SAND, BLOCK_TEXTURE_SAND, BLOCK_TEXTURE_SAND, BLOCK_TEXTURE_SAND, BLOCK_TEXTURE_SAND, BLOCK_TEXTURE_SAND }, // Sand
    { BLOCK_TEXTURE_STONE, BLOCK_TEXTURE_STONE, BLOCK_TEXTURE_STONE, BLOCK_TEXTURE_STONE, BLOCK_TEXTURE_STONE, BLOCK_TEXTURE_STONE }, // Stone
    { BLOCK_TEXTURE_COAL, BLOCK_TEXTURE_COAL, BLOCK_TEXTURE_COAL, BLOCK_TEXTURE_COAL, BLOCK_TEXTURE_COAL, BLOCK_TEXTURE_COAL }, // Coal
    { BLOCK_TEXTURE_GOLD, BLOCK_TEXTURE_GOLD, BLOCK_TEXTURE_GOLD, BLOCK_TEXTURE_GOLD, BLOCK_TEXTURE_GOLD, BLOCK_TEXTURE_GOLD }, // Goal
    { BLOCK_TEXTURE_TRUNK_TOP, BLOCK_TEXTURE_TRUNK_SIDE, BLOCK_TEXTURE_TRUNK_SIDE, BLOCK_TEXTURE_TRUNK_SIDE, BLOCK_TEXTURE_TRUNK_SIDE, BLOCK_TEXTURE_TRUNK_TOP }, // Trunk
    { BLOCK_TEXTURE_LEAVES, BLOCK_TEXTURE_LEAVES, BLOCK_TEXTURE_LEAVES, BLOCK_TEXTURE_LEAVES, BLOCK_TEXTURE_LEAVES, BLOCK_TEXTURE_LEAVES }, // Leaves
    { BLOCK_TEXTURE_CACTUS_TOP, BLOCK_TEXTURE_CACTUS_SIDE, BLOCK_TEXTURE_CACTUS_SIDE, BLOCK_TEXTURE_CACTUS_SIDE, BLOCK_TEXTURE_CACTUS_SIDE, BLOCK_TEXTURE_CACTUS_TOP } // Cactus
};





static int SEED;

static int hash[] = {208,34,231,213,32,248,233,56,161,78,24,140,71,48,140,254,245,255,247,247,40,
                     185,248,251,245,28,124,204,204,76,36,1,107,28,234,163,202,224,245,128,167,204,
                     9,92,217,54,239,174,173,102,193,189,190,121,100,108,167,44,43,77,180,204,8,81,
                     70,223,11,38,24,254,210,210,177,32,81,195,243,125,8,169,112,32,97,53,195,13,
                     203,9,47,104,125,117,114,124,165,203,181,235,193,206,70,180,174,0,167,181,41,
                     164,30,116,127,198,245,146,87,224,149,206,57,4,192,210,65,210,129,240,178,105,
                     228,108,245,148,140,40,35,195,38,58,65,207,215,253,65,85,208,76,62,3,237,55,89,
                     232,50,217,64,244,157,199,121,252,90,17,212,203,149,152,140,187,234,177,73,174,
                     193,100,192,143,97,53,145,135,19,103,13,90,135,151,199,91,239,247,33,39,145,
                     101,120,99,3,186,86,99,41,237,203,111,79,220,135,158,42,30,154,120,67,87,167,
                     135,176,183,191,253,115,184,21,233,58,129,233,142,39,128,211,118,137,139,255,
                     114,20,218,113,154,27,127,246,250,1,8,198,250,209,92,222,173,21,88,102,219};

int noise2(int x, int y)
{
    int tmp = hash[(y + SEED) % 256];
    return hash[(tmp + x) % 256];
}

float lin_inter(float x, float y, float s)
{
    return x + s * (y-x);
}

float smooth_inter(float x, float y, float s)
{
    return lin_inter(x, y, s * s * (3-2*s));
}

float noise2d(float x, float y)
{
    int x_int = x;
    int y_int = y;
    float x_frac = x - x_int;
    float y_frac = y - y_int;
    int s = noise2(x_int, y_int);
    int t = noise2(x_int+1, y_int);
    int u = noise2(x_int, y_int+1);
    int v = noise2(x_int+1, y_int+1);
    float low = smooth_inter(s, t, x_frac);
    float high = smooth_inter(u, v, x_frac);
    return smooth_inter(low, high, y_frac);
}

float perlin2d(float x, float y, float freq, int depth)
{
    float xa = x*freq;
    float ya = y*freq;
    float amp = 1.0;
    float fin = 0;
    float div = 0.0;

    int i;
    for(i=0; i<depth; i++)
    {
        div += 256 * amp;
        fin += noise2d(xa, ya) * amp;
        amp /= 2;
        xa *= 2;
        ya *= 2;
    }

    return fin/div;
}









void game_init() {
    if (!glfwInit()) {
        log_error("Can't init glfw");
    }
    atexit(glfwTerminate);
}

void game_framebuffer_size_callback(GLFWwindow* window, int width, int height) {
    Game* game = (Game*)glfwGetWindowUserPointer(window);
    game->width = width;
    game->height = height;

    game->camera->aspect = (float)width / (float)height;

    glViewport(0, 0, width, height);
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
            if (!game->isFullscreen) {
                game->isFullscreen = true;
                game->oldWidth = game->width;
                game->oldHeight = game->height;
                glfwSetWindowMonitor(window, primary_monitor, 0, 0, primary_screen->width, primary_screen->height, GLFW_DONT_CARE);
            } else {
                game->isFullscreen = false;
                glfwSetWindowMonitor(window, NULL, (primary_screen->width - game->oldWidth) / 2, (primary_screen->height - game->oldHeight) / 2, game->oldWidth, game->oldHeight, GLFW_DONT_CARE);
                glfwRestoreWindow(window);
            }
        }

        if (key == GLFW_KEY_ESCAPE && game->isCursorLocked) {
            game->isCursorLocked = false;
            glfwSetInputMode(window, GLFW_CURSOR, GLFW_CURSOR_NORMAL);
        }

        if ((mods & GLFW_MOD_CONTROL) != 0 && key == GLFW_KEY_I) {
            game->isWireframed = !game->isWireframed;
            glPolygonMode(GL_FRONT_AND_BACK, game->isWireframed ? GL_LINE : GL_FILL);
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

    if (button == GLFW_MOUSE_BUTTON_LEFT && action == GLFW_PRESS && !game->isCursorLocked) {
        glfwGetCursorPos(window, &lastX, &lastY);
        glfwSetInputMode(window, GLFW_CURSOR, GLFW_CURSOR_DISABLED);
        game->isCursorLocked = true;
    }
}

void game_cursor_position_callback(GLFWwindow* window, double xpos, double ypos) {
    Game* game = (Game*)glfwGetWindowUserPointer(window);
    if (game->isCursorLocked) {
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

Game* game_new(char* title, int width, int height) {
    glfwWindowHint(GLFW_CONTEXT_VERSION_MAJOR, 3);
    glfwWindowHint(GLFW_CONTEXT_VERSION_MINOR, 3);
    glfwWindowHint(GLFW_OPENGL_PROFILE, GLFW_OPENGL_CORE_PROFILE);
    #ifdef __APPLE__
        glfwWindowHint(GLFW_OPENGL_FORWARD_COMPAT, GL_TRUE);
    #endif

    Game* game = malloc(sizeof(Game));
    game->title = title;
    game->width = width;
    game->height = height;

    game->window = glfwCreateWindow(width, height, title, NULL, NULL);
    if (!game->window) {
        log_error("Can't create glfw window");
    }

    GLFWmonitor* primaryMonitor = glfwGetPrimaryMonitor();
    const GLFWvidmode* primaryScreen = glfwGetVideoMode(primaryMonitor);
    glfwSetWindowPos(game->window, (primaryScreen->width - width) / 2, (primaryScreen->height - height) / 2);

    glfwSetWindowSizeLimits(game->window, GAME_MIN_WIDTH, GAME_MIN_HEIGHT, GLFW_DONT_CARE, GLFW_DONT_CARE);
    glfwMakeContextCurrent(game->window);
    glfwSwapInterval(1);

    glfwSetWindowUserPointer(game->window, game);
    glfwSetFramebufferSizeCallback(game->window, game_framebuffer_size_callback);
    glfwSetKeyCallback(game->window, game_key_callback);
    glfwSetMouseButtonCallback(game->window, game_mouse_button_callback);
    glfwSetCursorPosCallback(game->window, game_cursor_position_callback);

    if (!gladLoadGLLoader((GLADloadproc)glfwGetProcAddress)) {
        log_error("Glad can't load OpenGL context");
    }

    log_info("Using an OpenGL %d.%d context", GLVersion.major, GLVersion.minor);

    return game;
}

void game_update(Game* game, float delta) {
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
    vector4_add(&game->camera->position, &update);

    camera_update_matrix(game->camera);
}

void game_render(Game* game) {
    (void)game;
}

BlockType generate_block(int x, int y, int z) {
    int height = perlin2d((float)(x + 10000) / 16, (float)(z + 10000) / 16, 1, 1) * 16;

    int weatness = perlin2d((float)(x + 100000000) / 16, (float)(z + 100000000) / 16, 1, 1) * 3;

    int sea_level = 0;
    if (y <= sea_level) {
        return BLOCK_TYPE_WATER;
    }

    if (weatness > 0.5) {
        if (y == height) {
            return BLOCK_TYPE_GRASS;
        }

        if (y >= -8 + height && y < height) {
            return BLOCK_TYPE_DIRT;
        }
    } else {
        if (y >= -8 + height && y <= height) {
            return BLOCK_TYPE_SAND;
        }
    }

    if (y >= -64 + height && y < -8 + height) {
        if ((rand() % 10) == 0) {
            return BLOCK_TYPE_COAL;
        }

        if ((rand() % 20) == 0) {
            return BLOCK_TYPE_GOLD;
        }

        return BLOCK_TYPE_STONE;
    }

    return BLOCK_TYPE_AIR;
}

Chunk *generate_chunk(Game *game, int chunk_x, int chunk_y, int chunk_z) {
    srand(chunk_x + chunk_y + chunk_z);

    // log_info("Create chunk %d %d %d", chunk_x, chunk_y, chunk_z);

    Chunk *chunk = malloc(sizeof(Chunk));
    chunk->x = chunk_x;
    chunk->y = chunk_y;
    chunk->z = chunk_z;
    chunk->data = malloc(CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE);
    chunk->visible = NULL;

    if (game->chunks_size == (sizeof(game->chunks) / sizeof(Chunk*))) {
        game->chunks_size = 0;
    }
    if (game->chunks[game->chunks_size] != NULL) {
        Chunk* otherChunk = game->chunks[game->chunks_size];
        free(otherChunk->data);
        if (otherChunk->visible != NULL) {
            free(otherChunk->visible);
        }
        free(otherChunk);
    }
    game->chunks[game->chunks_size] = chunk;
    game->chunks_size++;

    for (int block_z = 0; block_z < CHUNK_SIZE; block_z++) {
        for (int block_y = 0; block_y < CHUNK_SIZE; block_y++) {
            for (int block_x = 0; block_x < CHUNK_SIZE; block_x++) {
                int block_position = block_z * CHUNK_SIZE * CHUNK_SIZE + block_y * CHUNK_SIZE + block_x;

                // Change block
                BlockType blockType = generate_block(chunk_x * CHUNK_SIZE + block_x, chunk_y * CHUNK_SIZE + block_y, chunk_z * CHUNK_SIZE + block_z);
                if (blockType == BLOCK_TYPE_AIR) {
                    if (chunk->data[block_position] != BLOCK_TYPE_TRUNK && chunk->data[block_position] != BLOCK_TYPE_LEAVES && chunk->data[block_position] != BLOCK_TYPE_CACTUS) {
                        chunk->data[block_position] = BLOCK_TYPE_AIR;
                    }
                } else {
                    chunk->data[block_position] = blockType;
                }

                // Place tree
                if (
                    block_x >= 2 && block_z >= 2 && block_x < CHUNK_SIZE - 3 && block_z < CHUNK_SIZE - 3 && block_y < CHUNK_SIZE - 7 &&
                    blockType == BLOCK_TYPE_GRASS && (rand() % 100) == 0
                ) {
                    for (int tree_y = 1; tree_y < 6; tree_y++) {
                        if (tree_y >= 3) {
                            for (int leave_z = -1; leave_z <= 1; leave_z++) {
                                for (int leave_x = -1; leave_x <= 1; leave_x++) {
                                    chunk->data[(block_z + leave_z) * CHUNK_SIZE * CHUNK_SIZE + (block_y + tree_y) * CHUNK_SIZE + (block_x + leave_x)] =
                                        leave_z == 0 && leave_x == 0 && tree_y != 5 ? BLOCK_TYPE_TRUNK : BLOCK_TYPE_LEAVES;
                                }
                            }
                        } else {
                            chunk->data[block_z * CHUNK_SIZE * CHUNK_SIZE + (block_y + tree_y) * CHUNK_SIZE + block_x] = BLOCK_TYPE_TRUNK;
                        }
                    }
                }

                // Place cactus
                if (block_y < CHUNK_SIZE - 6 && blockType == BLOCK_TYPE_SAND && (rand() % 200) == 0) {
                    for (int cactus_y = 1; cactus_y < 5; cactus_y++) {
                        chunk->data[block_z * CHUNK_SIZE * CHUNK_SIZE + (block_y + cactus_y) * CHUNK_SIZE + block_x] = BLOCK_TYPE_CACTUS;
                    }
                }
            }
        }
    }

    return chunk;
}

Chunk *game_get_chunk(Game *game, int chunk_x, int chunk_y, int chunk_z);

void chunk_calculate_visible(Game *game, Chunk *chunk) {
    // log_info("Calculate visible %d %d %d", chunk->x, chunk->y, chunk->z);

    if (chunk->visible == NULL) {
        chunk->visible = malloc(CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE);
    }

    // Calc visible
    for (int block_z = 0; block_z < CHUNK_SIZE; block_z++) {
        for (int block_y = 0; block_y < CHUNK_SIZE; block_y++) {
            for (int block_x = 0; block_x < CHUNK_SIZE; block_x++) {
                int block_index = block_z * CHUNK_SIZE * CHUNK_SIZE + block_y * CHUNK_SIZE + block_x;

                BlockType blockType = chunk->data[block_index];
                if (blockType == BLOCK_TYPE_AIR) {
                    chunk->visible[block_index] = 0;
                    continue;
                }

                // Check above block
                if (block_y == 0) {
                    Chunk *otherChunk = game_get_chunk(game, chunk->x, chunk->y - 1, chunk->z);
                    BlockType blockType = otherChunk->data[block_z * CHUNK_SIZE * CHUNK_SIZE + (CHUNK_SIZE - 1) * CHUNK_SIZE + block_x];
                    if (blockType == BLOCK_TYPE_AIR) {
                        chunk->visible[block_index] = 1;
                        continue;
                    }
                } else {
                    BlockType blockType = chunk->data[block_z * CHUNK_SIZE * CHUNK_SIZE + (block_y - 1) * CHUNK_SIZE + block_x];
                    if (blockType == BLOCK_TYPE_AIR) {
                        chunk->visible[block_index] = 1;
                        continue;
                    }
                }

                // Check below block
                if (block_y == CHUNK_SIZE - 1) {
                    Chunk *otherChunk = game_get_chunk(game, chunk->x, chunk->y + 1, chunk->z);
                    BlockType blockType = otherChunk->data[block_z * CHUNK_SIZE * CHUNK_SIZE + 0 * CHUNK_SIZE + block_x];
                    if (blockType == BLOCK_TYPE_AIR) {
                        chunk->visible[block_index] = 1;
                        continue;
                    }
                } else {
                    BlockType blockType = chunk->data[block_z * CHUNK_SIZE * CHUNK_SIZE + (block_y + 1) * CHUNK_SIZE + block_x];
                    if (blockType == BLOCK_TYPE_AIR) {
                        chunk->visible[block_index] = 1;
                        continue;
                    }
                }

                // Check left block
                if (block_x == 0) {
                    Chunk *otherChunk = game_get_chunk(game, chunk->x - 1, chunk->y, chunk->z);
                    BlockType blockType = otherChunk->data[block_z * CHUNK_SIZE * CHUNK_SIZE + block_y * CHUNK_SIZE + (CHUNK_SIZE - 1)];
                    if (blockType == BLOCK_TYPE_AIR) {
                        chunk->visible[block_index] = 1;
                        continue;
                    }
                } else {
                    BlockType blockType = chunk->data[block_z * CHUNK_SIZE * CHUNK_SIZE + block_y * CHUNK_SIZE + (block_x - 1)];
                    if (blockType == BLOCK_TYPE_AIR) {
                        chunk->visible[block_index] = 1;
                        continue;
                    }
                }

                // Check right block
                if (block_x == CHUNK_SIZE - 1) {
                    Chunk *otherChunk = game_get_chunk(game, chunk->x + 1, chunk->y, chunk->z);
                    BlockType blockType = otherChunk->data[block_z * CHUNK_SIZE * CHUNK_SIZE + block_y * CHUNK_SIZE + 0];
                    if (blockType == BLOCK_TYPE_AIR) {
                        chunk->visible[block_index] = 1;
                        continue;
                    }
                } else {
                    BlockType blockType = chunk->data[block_z * CHUNK_SIZE * CHUNK_SIZE + block_y * CHUNK_SIZE + (block_x + 1)];
                    if (blockType == BLOCK_TYPE_AIR) {
                        chunk->visible[block_index] = 1;
                        continue;
                    }
                }

                // Check front block
                if (block_z == 0) {
                    Chunk *otherChunk = game_get_chunk(game, chunk->x, chunk->y, chunk->z - 1);
                    BlockType blockType = otherChunk->data[(CHUNK_SIZE - 1) * CHUNK_SIZE * CHUNK_SIZE + block_y * CHUNK_SIZE + block_x];
                    if (blockType == BLOCK_TYPE_AIR) {
                        chunk->visible[block_index] = 1;
                        continue;
                    }
                } else {
                    BlockType blockType = chunk->data[(block_z - 1) * CHUNK_SIZE * CHUNK_SIZE + block_y * CHUNK_SIZE + block_x];
                    if (blockType == BLOCK_TYPE_AIR) {
                        chunk->visible[block_index] = 1;
                        continue;
                    }
                }

                // Check back block
                if (block_z == CHUNK_SIZE - 1) {
                    Chunk *otherChunk = game_get_chunk(game, chunk->x, chunk->y, chunk->z + 1);
                    BlockType blockType = otherChunk->data[0 * CHUNK_SIZE * CHUNK_SIZE + block_y * CHUNK_SIZE + block_x];
                    if (blockType == BLOCK_TYPE_AIR) {
                        chunk->visible[block_index] = 1;
                        continue;
                    }
                } else {
                    BlockType blockType = chunk->data[(block_z + 1) * CHUNK_SIZE * CHUNK_SIZE + block_y * CHUNK_SIZE + block_x];
                    if (blockType == BLOCK_TYPE_AIR) {
                        chunk->visible[block_index] = 1;
                        continue;
                    }
                }

                // Not visible
                chunk->visible[block_index] = 0;
            }
        }
    }
}

Chunk *game_get_chunk(Game *game, int chunk_x, int chunk_y, int chunk_z) {
    for (int i = 0; i < game->chunks_size; i++) {
        Chunk* chunk = game->chunks[i];
        if (chunk->x == chunk_x && chunk->y == chunk_y && chunk->z == chunk_z) {
            return chunk;
        }
    }

    return generate_chunk(game, chunk_x, chunk_y, chunk_z);
}

void game_start(Game* game) {
    glEnable(GL_DEPTH_TEST);

    GLuint vertex_array;
    glGenVertexArrays(1, &vertex_array);
    glBindVertexArray(vertex_array);

    float vertices[] = {
        // Vertex position, Texture position, Texture face
        -0.5, -0.5, -0.5,  0, 0,  5,
        0.5, -0.5, -0.5,  1, 0,  5,
        0.5,  0.5, -0.5,  1, 1,  5,
        0.5,  0.5, -0.5,  1, 1,  5,
        -0.5,  0.5, -0.5,  0, 1,  5,
        -0.5, -0.5, -0.5,  0, 0,  5,

        -0.5, -0.5,  0.5,  0, 0,  0,
        0.5, -0.5,  0.5,  1, 0,  0,
        0.5,  0.5,  0.5,  1, 1,  0,
        0.5,  0.5,  0.5,  1, 1,  0,
        -0.5,  0.5,  0.5,  0, 1,  0,
        -0.5, -0.5,  0.5,  0, 0,  0,

        -0.5,  0.5,  0.5,  0, 0,  3,
        -0.5,  0.5, -0.5,  0, 1,  3,
        -0.5, -0.5, -0.5,  1, 1,  3,
        -0.5, -0.5, -0.5,  1, 1,  3,
        -0.5, -0.5,  0.5,  1, 0,  3,
        -0.5,  0.5,  0.5,  0, 0,  3,

        0.5,  0.5,  0.5,  1, 0,  1,
        0.5,  0.5, -0.5,  1, 1,  1,
        0.5, -0.5, -0.5,  0, 1,  1,
        0.5, -0.5, -0.5,  0, 1,  1,
        0.5, -0.5,  0.5,  0, 0,  1,
        0.5,  0.5,  0.5,  1, 0,  1,

        -0.5, -0.5, -0.5,  0, 1,  4,
        0.5, -0.5, -0.5,  1, 1,  4,
        0.5, -0.5,  0.5,  1, 0,  4,
        0.5, -0.5,  0.5,  1, 0,  4,
        -0.5, -0.5,  0.5,  0, 0,  4,
        -0.5, -0.5, -0.5,  0, 1,  4,

        -0.5,  0.5, -0.5,  1, 1,  2,
        0.5,  0.5, -0.5,  0, 1,  2,
        0.5,  0.5,  0.5,  0, 0,  2,
        0.5,  0.5,  0.5,  0, 0,  2,
        -0.5,  0.5,  0.5,  1, 0,  2,
        -0.5,  0.5, -0.5,  1, 1,  2
    };

    GLuint vertex_buffer;
    glGenBuffers(1, &vertex_buffer);
    glBindBuffer(GL_ARRAY_BUFFER, vertex_buffer);
    glBufferData(GL_ARRAY_BUFFER, sizeof(vertices), vertices, GL_STATIC_DRAW);

    // Load shader
    Shader* blockShader = shader_new("assets/shaders/block.vert", "assets/shaders/block.frag");

    // Load Texture
    TextureAtlas* blocksTextureAtlas = texture_atlas_new("assets/images/blocks.png", 128);

    // Create camera
    game->camera = camera_new(radians(45), (float)game->width / (float)game->height, 0.1, 1000);
    game->camera->position.x = CHUNK_SIZE / 2;
    game->camera->position.y = 1.5 + CHUNK_SIZE;
    game->camera->position.z = CHUNK_SIZE / 2;
    camera_update_matrix(game->camera);

    game->isMovingForward = false;
    game->isMovingLeft = false;
    game->isMovingRight = false;
    game->isMovingBackward = false;
    game->isMovingUp = false;
    game->isMovingDown = false;

    game->velocity.x = 0;
    game->velocity.y = 0;
    game->velocity.z = 0;

    // Setup chunks cache
    SEED = 0;
    game->render_distance = 3;


    for (size_t i = 0; i < sizeof(game->chunks) / sizeof(Chunk*); i++) {
        game->chunks[i] = NULL;
    }
    game->chunks_size = 0;

    double lastTime = 0;
    double frameTime = 0;
    int frame = 0;
    while (!glfwWindowShouldClose(game->window)) {
        glfwPollEvents();

        // Update
        double time = glfwGetTime();

        frame++;
        if (time - frameTime >= 1.0) {
            system("clear");
            log_info("Fps %d", frame);
            frame = 0;
            frameTime = time;
        }

        float delta = (float)(time - lastTime);
        lastTime = time;
        game_update(game, delta);

        // Render
        glClearColor(176.f / 255.f, 233.f / 255.f, 252.f / 255.f, 1);
        glClear(GL_COLOR_BUFFER_BIT | GL_DEPTH_BUFFER_BIT);

        glUseProgram(blockShader->program);

        glBindTexture(GL_TEXTURE_2D_ARRAY, blocksTextureAtlas->textureArray);

        glBindVertexArray(vertex_array);

        glUniformMatrix4fv(blockShader->projection_matrix_uniform, 1, GL_FALSE, (const GLfloat*)&game->camera->projectionMatrix);
        glUniformMatrix4fv(blockShader->view_matrix_uniform, 1, GL_FALSE, (const GLfloat*)&game->camera->viewMatrix);

        int player_chunk_x = floor(game->camera->position.x / (float)CHUNK_SIZE);
        int player_chunk_y = floor(game->camera->position.y / (float)CHUNK_SIZE);
        int player_chunk_z = floor(game->camera->position.z / (float)CHUNK_SIZE);

        // system("clear");
        // log_info("Camera %.3g %.3g %.3g", game->camera->position.x, game->camera->position.y, game->camera->position.z);
        // log_info("Chunk %d %d %d", player_chunk_x, player_chunk_y, player_chunk_z);

        Matrix4 rotationMatrix;
        matrix4_rotate_x(&rotationMatrix, radians(-90));

        for (int chunk_z = player_chunk_z - game->render_distance; chunk_z <= player_chunk_z + game->render_distance; chunk_z++) {
            for (int chunk_y = player_chunk_y - game->render_distance; chunk_y <= player_chunk_y + game->render_distance; chunk_y++) {
                for (int chunk_x = player_chunk_x - game->render_distance; chunk_x <= player_chunk_x + game->render_distance; chunk_x++) {

                    bool isVisible = true;

                    // Vector4 corners[8] = {
                    //     { chunk_x, chunk_y, chunk_z, 1 },
                    //     { chunk_x + CHUNK_SIZE, chunk_y, chunk_z, 1 },
                    //     { chunk_x, chunk_y + CHUNK_SIZE, chunk_z, 1 },
                    //     { chunk_x + CHUNK_SIZE, chunk_y + CHUNK_SIZE, chunk_z, 1 },

                    //     { chunk_x, chunk_y, chunk_z + CHUNK_SIZE, 1 },
                    //     { chunk_x + CHUNK_SIZE, chunk_y, chunk_z + CHUNK_SIZE, 1 },
                    //     { chunk_x, chunk_y + CHUNK_SIZE, chunk_z + CHUNK_SIZE, 1 },
                    //     { chunk_x + CHUNK_SIZE, chunk_y + CHUNK_SIZE, chunk_z + CHUNK_SIZE, 1 }
                    // };

                    // for (size_t i = 0; i < sizeof(corners) / sizeof(Vector4); i++) {
                    //     vector4_mul(&corners[i], &game->camera->viewMatrix);
                    //     vector4_mul(&corners[i], &game->camera->projectionMatrix);

                    //     #define within(a, b, c) ((a) >= (b) && (b) <= (c))
                    //     if (
                    //         within(-corners[i].w, corners[i].x, corners[i].w) &&
                    //         within(-corners[i].w, corners[i].y, corners[i].w) &&
                    //         within(0, corners[i].z, corners[i].w)
                    //     ) {
                    //         isVisible = true;
                    //         break;
                    //     }
                    // }

                    if (isVisible) {
                        Chunk *chunk = game_get_chunk(game, chunk_x, chunk_y, chunk_z);

                        if (chunk->visible == NULL) {
                            chunk_calculate_visible(game, chunk);
                        }

                        for (int block_z = 0; block_z < CHUNK_SIZE; block_z++) {
                            for (int block_y = 0; block_y < CHUNK_SIZE; block_y++) {
                                for (int block_x = 0; block_x < CHUNK_SIZE; block_x++) {
                                    int block_index = block_z * CHUNK_SIZE * CHUNK_SIZE + block_y * CHUNK_SIZE + block_x;
                                    if (chunk->visible[block_index]) {
                                        BlockType blockType = chunk->data[block_index];

                                        Matrix4 modelMatrix;
                                        Vector4 blockPosition = {
                                            chunk_x * CHUNK_SIZE + block_x,
                                            chunk_y * CHUNK_SIZE + block_y,
                                            -(chunk_z * CHUNK_SIZE + block_z),
                                            1
                                        };
                                        matrix4_translate(&modelMatrix, &blockPosition);
                                        matrix4_mul(&modelMatrix, &rotationMatrix);

                                        glUniformMatrix4fv(blockShader->model_matrix_uniform, 1, GL_FALSE, (const GLfloat*)&modelMatrix.m11);

                                        glUniform1iv(blockShader->texture_indexes_uniform, 6, (const GLint *)&BLOCK_FACES[blockType]);

                                        glDrawArrays(GL_TRIANGLES, 0, 36);
                                    }
                                }
                            }
                        }
                    } else {
                        // log_info("Skip chunk");
                    }
                }
            }
        }

        glfwSwapBuffers(game->window);
    }

    shader_free(blockShader);

    texture_atlas_free(blocksTextureAtlas);
}

void game_free(Game* game) {
    glfwDestroyWindow(game->window);
}
