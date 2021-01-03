// PlaatCraft - Shader

#include "shader.h"
#include <stdlib.h>
#include "utils.h"
#include "log.h"

Shader* shader_new(char* vertex_path, char* fragment_path) {
    Shader* shader = malloc(sizeof(Shader));

    // Read and compile vertex shader
    const char* vertex_shader_text = file_read(vertex_path);
    GLuint vertex_shader = glCreateShader(GL_VERTEX_SHADER);
    glShaderSource(vertex_shader, 1, &vertex_shader_text, NULL);
    glCompileShader(vertex_shader);
    free((char*)vertex_shader_text);

    int success;
    char infoLog[512];
    glGetShaderiv(vertex_shader, GL_COMPILE_STATUS, &success);
    if (!success) {
        glGetShaderInfoLog(vertex_shader, sizeof(infoLog), NULL, infoLog);
        log_error("Can't compile vertex shader: %s\n", infoLog);
    }

    // Compile fragment shader
    const char* fragment_shader_text = file_read(fragment_path);
    GLuint fragment_shader = glCreateShader(GL_FRAGMENT_SHADER);
    glShaderSource(fragment_shader, 1, &fragment_shader_text, NULL);
    glCompileShader(fragment_shader);
    free((char*)fragment_shader_text);

    glGetShaderiv(fragment_shader, GL_COMPILE_STATUS, &success);
    if (!success) {
        glGetShaderInfoLog(fragment_shader, sizeof(infoLog), NULL, infoLog);
        log_error("Can't compile fragment shader: %s\n", infoLog);
    }

    // Link program
    shader->program = glCreateProgram();
    glAttachShader(shader->program, vertex_shader);
    glAttachShader(shader->program, fragment_shader);
    glLinkProgram(shader->program);

    glGetProgramiv(shader->program, GL_LINK_STATUS, &success);
    if(!success) {
        glGetProgramInfoLog(shader->program, sizeof(infoLog), NULL, infoLog);
        log_error("Can't link program: %s\n", infoLog);
    }

    // Delete shaders
    glDeleteShader(vertex_shader);
    glDeleteShader(fragment_shader);

    // Get attributes
    shader->position_attribute = glGetAttribLocation(shader->program, "a_position");
    glVertexAttribPointer(shader->position_attribute, 3, GL_FLOAT, GL_FALSE, 6 * sizeof(float), (void*)0);
    glEnableVertexAttribArray(shader->position_attribute);

    shader->texture_position_attribute = glGetAttribLocation(shader->program, "a_texture_position");
    glVertexAttribPointer(shader->texture_position_attribute, 2, GL_FLOAT, GL_FALSE, 6 * sizeof(float), (void*)(3 * sizeof(float)));
    glEnableVertexAttribArray(shader->texture_position_attribute);

    shader->texture_face_attribute = glGetAttribLocation(shader->program, "a_texture_face");
    glVertexAttribPointer(shader->texture_face_attribute, 2, GL_FLOAT, GL_FALSE, 6 * sizeof(float), (void*)(5 * sizeof(float)));
    glEnableVertexAttribArray(shader->texture_face_attribute);


    // Get uniforms
    shader->model_matrix_uniform = glGetUniformLocation(shader->program, "u_model_matrix");
    shader->view_matrix_uniform = glGetUniformLocation(shader->program, "u_view_matrix");
    shader->projection_matrix_uniform = glGetUniformLocation(shader->program, "u_projection_matrix");
    shader->texture_indexes_uniform = glGetUniformLocation(shader->program, "u_texture_indexes");

    return shader;
}

void shader_free(Shader* shader) {
    // Delete program
    glDeleteProgram(shader->program);
}
