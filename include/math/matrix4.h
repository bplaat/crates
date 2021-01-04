// PlaatCraft - Matrix4 Math Header

#ifndef MATRIX4_H
#define MATRIX4_H

#include "utils.h"
#include "math/rect.h"

#ifndef NO_SSIMD
typedef struct _MM_ALIGN16 Matrix4 {
#else
typedef struct Matrix4 {
#endif
    float m11, m12, m13, m14;
    float m21, m22, m23, m24;
    float m31, m32, m33, m34;
    float m41, m42, m43, m44;
} Matrix4;

#include "math/vector4.h"

void matrix4_identity(Matrix4* matrix);

void matrix4_perspective(Matrix4* matrix, float fov, float aspect, float near, float far);

void matrix4_translate(Matrix4* matrix, Vector4* position);

void matrix4_rotate_x(Matrix4* matrix, float x);

void matrix4_rotate_y(Matrix4* matrix, float y);

void matrix4_rotate_z(Matrix4* matrix, float z);

void matrix4_scale(Matrix4* matrix, Vector4* scale);

void matrix4_mul(Matrix4* matrix, Matrix4* rhs);

#endif
