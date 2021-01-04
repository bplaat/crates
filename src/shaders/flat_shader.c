// PlaatCraft - Flat Shader

#include "shaders/flat_shader.h"
#include <stdlib.h>
#include "utils.h"
#include "log.h"

FlatShader* flat_shader_new() {
    FlatShader* flat_shader = malloc(sizeof(Shader));
    flat_shader->shader = shader_new("assets/shaders/flat.vert", "assets/shaders/flat.frag");

    flat_shader->plane = plane_new();

    // Get attributes
    flat_shader->position_attribute = glGetAttribLocation(flat_shader->shader->program, "a_position");
    glVertexAttribPointer(flat_shader->position_attribute, 3, GL_FLOAT, GL_FALSE, 6 * sizeof(float), (void*)0);
    glEnableVertexAttribArray(flat_shader->position_attribute);

    flat_shader->texture_position_attribute = glGetAttribLocation(flat_shader->shader->program, "a_texture_position");
    glVertexAttribPointer(flat_shader->texture_position_attribute, 2, GL_FLOAT, GL_FALSE, 6 * sizeof(float), (void*)(3 * sizeof(float)));
    glEnableVertexAttribArray(flat_shader->texture_position_attribute);

    // Get uniforms
    flat_shader->matrix_uniform = glGetUniformLocation(flat_shader->shader->program, "u_matrix");

    return flat_shader;
}

void flat_shader_use(FlatShader* flat_shader) {
    glUseProgram(flat_shader->shader->program);
    plane_use(flat_shader->plane);
}

void flat_shader_free(FlatShader* flat_shader) {
    shader_free(flat_shader->shader);
    free(flat_shader);
}
