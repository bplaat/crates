// PlaatCraft - Block Shader

#include "shaders/block_shader.h"
#include <stdlib.h>
#include "utils.h"
#include "log.h"

BlockShader* block_shader_new(void) {
    BlockShader* block_shader = malloc(sizeof(BlockShader));
    block_shader->shader = shader_new("assets/shaders/block.vert", "assets/shaders/block.frag");
    block_shader->block = block_new();
    block_shader_enable(block_shader);

    // Get attributes
    block_shader->position_attribute = glGetAttribLocation(block_shader->shader->program, "a_position");
    glVertexAttribPointer(block_shader->position_attribute, 3, GL_FLOAT, GL_FALSE, 6 * sizeof(float), 0);
    glEnableVertexAttribArray(block_shader->position_attribute);

    block_shader->texture_position_attribute = glGetAttribLocation(block_shader->shader->program, "a_texture_position");
    glVertexAttribPointer(block_shader->texture_position_attribute, 2, GL_FLOAT, GL_FALSE, 6 * sizeof(float), (void*)(3 * sizeof(float)));
    glEnableVertexAttribArray(block_shader->texture_position_attribute);

    block_shader->texture_face_attribute = glGetAttribLocation(block_shader->shader->program, "a_texture_face");
    glVertexAttribPointer(block_shader->texture_face_attribute, 1, GL_FLOAT, GL_FALSE, 6 * sizeof(float), (void*)(5 * sizeof(float)));
    glEnableVertexAttribArray(block_shader->texture_face_attribute);

    // Get uniforms
    block_shader->model_matrix_uniform = glGetUniformLocation(block_shader->shader->program, "u_model_matrix");
    block_shader->camera_matrix_uniform = glGetUniformLocation(block_shader->shader->program, "u_camera_matrix");
    block_shader->projection_matrix_uniform = glGetUniformLocation(block_shader->shader->program, "u_projection_matrix");
    block_shader->is_flad_shaded_uniform = glGetUniformLocation(block_shader->shader->program, "u_is_flat_shaded");
    block_shader->texture_indexes_uniform = glGetUniformLocation(block_shader->shader->program, "u_texture_indexes");

    // Disable shader
    block_shader_disable(block_shader);

    return block_shader;
}

void block_shader_enable(BlockShader* block_shader) {
    shader_enable(block_shader->shader);
    block_enable(block_shader->block);
}

void block_shader_disable(BlockShader* block_shader) {
    block_disable(block_shader->block);
}

void block_shader_free(BlockShader* block_shader) {
    block_free(block_shader->block);
    shader_free(block_shader->shader);
    free(block_shader);
}
