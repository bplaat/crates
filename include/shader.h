// PlaatCraft - Shader Header

#ifndef SHADER_H
#define SHADER_H

#include "glad.h"

typedef struct Shader {
    GLuint program;

    GLint position_attribute;
    GLint texture_position_attribute;
    GLint texture_face_attribute;

    GLint model_matrix_uniform;
    GLint view_matrix_uniform;
    GLint projection_matrix_uniform;
    GLint texture_indexes_uniform;
} Shader;

Shader* shader_new(char* vertex_path, char* fragment_path);

void shader_free(Shader* shader);

#endif
