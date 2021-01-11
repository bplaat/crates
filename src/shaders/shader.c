// PlaatCraft - Shader

#include "shaders/shader.h"
#include <stdlib.h>
#include "utils.h"
#include "log.h"

Shader* shader_new(char* vertex_path, char* fragment_path) {
    Shader* shader = malloc(sizeof(Shader));
    shader->vertex_path = vertex_path;
    shader->fragment_path = fragment_path;

    // Read and compile vertex shader
    const char* vertex_shader_source = file_read(vertex_path);
    GLuint vertex_shader = glCreateShader(GL_VERTEX_SHADER);
    glShaderSource(vertex_shader, 1, &vertex_shader_source, NULL);
    glCompileShader(vertex_shader);
    free((char*)vertex_shader_source);

    int success;
    char info_log[512];
    glGetShaderiv(vertex_shader, GL_COMPILE_STATUS, &success);
    if (!success) {
        glGetShaderInfoLog(vertex_shader, sizeof(info_log), NULL, info_log);
        log_error("Can't compile vertex shader %s:", vertex_path);
        log_error("%s", info_log);
    }

    //  Read and compile fragment shader
    const char* fragment_shader_source = file_read(fragment_path);
    GLuint fragment_shader = glCreateShader(GL_FRAGMENT_SHADER);
    glShaderSource(fragment_shader, 1, &fragment_shader_source, NULL);
    glCompileShader(fragment_shader);
    free((char*)fragment_shader_source);

    glGetShaderiv(fragment_shader, GL_COMPILE_STATUS, &success);
    if (!success) {
        glGetShaderInfoLog(fragment_shader, sizeof(info_log), NULL, info_log);
        log_error("Can't compile fragment shader %s:", fragment_path);
        log_error("%s", info_log);
    }

    // Link program
    shader->program = glCreateProgram();
    glAttachShader(shader->program, vertex_shader);
    glAttachShader(shader->program, fragment_shader);
    glLinkProgram(shader->program);

    glGetProgramiv(shader->program, GL_LINK_STATUS, &success);
    if(!success) {
        glGetProgramInfoLog(shader->program, sizeof(info_log), NULL, info_log);
        log_error("Can't link program %s and %s:", vertex_path, fragment_path);
        log_error("%s", info_log);
    }

    // Delete shaders
    glDeleteShader(vertex_shader);
    glDeleteShader(fragment_shader);

    return shader;
}

void shader_enable(Shader* shader) {
    // Use program
    glUseProgram(shader->program);
}

void shader_free(Shader* shader) {
    // Delete program
    glDeleteProgram(shader->program);
}
