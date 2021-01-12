// PlaatCraft - Flat Shader Header

#ifndef FLAT_SHADER_H
#define FLAT_SHADER_H

#include "shaders/shader.h"
#include "geometry/plane.h"

typedef struct FlatShader {
    Shader* shader;

    Plane* plane;

    GLint position_attribute;
    GLint texture_position_attribute;

    GLint model_matrix_uniform;
    GLint projection_matrix_uniform;
    GLint is_textured_uniform;
    GLint color_uniform;
} FlatShader;

FlatShader* flat_shader_new();

void flat_shader_enable(FlatShader* flat_shader);

void flat_shader_disable(FlatShader* flat_shader);

void flat_shader_free(FlatShader* flat_shader);

#endif
