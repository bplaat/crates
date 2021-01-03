// PlaatCraft - Block

#include "block.h"
#include <math.h>

BlockTexture BLOCK_TEXTURE_FACES[BLOCK_TYPE_SIZE][6] = {
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

float BLOCK_VERTICES[] = {
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

GLuint block_vertex_array;

GLuint block_vertex_buffer;

void block_init() {
    glGenVertexArrays(1, &block_vertex_array);
    glBindVertexArray(block_vertex_array);

    glGenBuffers(1, &block_vertex_buffer);
    glBindBuffer(GL_ARRAY_BUFFER, block_vertex_buffer);
    glBufferData(GL_ARRAY_BUFFER, sizeof(BLOCK_VERTICES), BLOCK_VERTICES, GL_STATIC_DRAW);
}

void block_free() {
    glDeleteVertexArrays(1, &block_vertex_array);
    glDeleteBuffers(1, &block_vertex_buffer);
}
