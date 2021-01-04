// PlaatCraft - Flat Shader

#include "shaders/flat_shader.h"
#include <stdlib.h>
#include "utils.h"
#include "log.h"

FlatShader* flat_shader_new() {
    FlatShader* flat_shader = malloc(sizeof(Shader));
    flat_shader->shader = shader_new("assets/shaders/flat.vert", "assets/shaders/flat.frag");
    flat_shader->plane = plane_new();
    flat_shader_enable(flat_shader);

    // Get attributes
    flat_shader->position_attribute = glGetAttribLocation(flat_shader->shader->program, "a_position");
    glVertexAttribPointer(flat_shader->position_attribute, 2, GL_FLOAT, GL_FALSE, 4 * sizeof(float), 0);
    glEnableVertexAttribArray(flat_shader->position_attribute);

    flat_shader->texture_position_attribute = glGetAttribLocation(flat_shader->shader->program, "a_texture_position");
    glVertexAttribPointer(flat_shader->texture_position_attribute, 2, GL_FLOAT, GL_FALSE, 4 * sizeof(float), (void*)(2 * sizeof(float)));
    glEnableVertexAttribArray(flat_shader->texture_position_attribute);

    // Get uniforms
    flat_shader->model_matrix_uniform = glGetUniformLocation(flat_shader->shader->program, "u_model_matrix");
    flat_shader->projection_matrix_uniform = glGetUniformLocation(flat_shader->shader->program, "u_projection_matrix");

    // Disable shader
    flat_shader_disable(flat_shader);

    return flat_shader;
}

void flat_shader_enable(FlatShader* flat_shader) {
    shader_enable(flat_shader->shader);
    plane_enable(flat_shader->plane);
}

void flat_shader_disable(FlatShader* flat_shader) {
    plane_disable(flat_shader->plane);
}

void flat_shader_free(FlatShader* flat_shader) {
    shader_free(flat_shader->shader);
    free(flat_shader);
}
