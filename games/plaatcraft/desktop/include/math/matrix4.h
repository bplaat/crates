// PlaatCraft - Matrix4 Math Header

#ifndef MATRIX4_H
#define MATRIX4_H

#include "utils.h"

typedef __attribute__((aligned(16))) struct Matrix4 {
    float elements[16];
} Matrix4;

#include "math/vector4.h"

void matrix4_identity(Matrix4* matrix);

void matrix4_perspective(Matrix4* matrix, float fov, float aspect, float _near, float _far);

void matrix4_translate(Matrix4* matrix, Vector4* position);

void matrix4_rotate_x(Matrix4* matrix, float x);

void matrix4_rotate_y(Matrix4* matrix, float y);

void matrix4_rotate_z(Matrix4* matrix, float z);

void matrix4_scale(Matrix4* matrix, Vector4* scale);

void matrix4_flat_projection(Matrix4* matrix, int viewport_width, int viewport_height);

void matrix4_flat_rect(Matrix4* matrix, int x, int y, int width, int height);

void matrix4_mul_matrix4(Matrix4* matrix, Matrix4* rhs);

#endif
