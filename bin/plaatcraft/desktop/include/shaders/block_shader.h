// PlaatCraft - Block Shader Header

#ifndef BLOCK_SHADER_H
#define BLOCK_SHADER_H

#include "shaders/shader.h"
#include "geometry/block.h"

typedef struct BlockShader {
    Shader* shader;

    Block* block;

    GLint position_attribute;
    GLint texture_position_attribute;
    GLint texture_face_attribute;

    GLint model_matrix_uniform;
    GLint view_matrix_uniform;
    GLint projection_matrix_uniform;
    GLint is_lighted_uniform;
    GLint is_flad_shaded_uniform;
    GLint texture_indexes_uniform;
} BlockShader;

BlockShader* block_shader_new(void);

void block_shader_enable(BlockShader* block_shader);

void block_shader_disable(BlockShader* block_shader);

void block_shader_free(BlockShader* block_shader);

#endif
