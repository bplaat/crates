// PlaatCraft - Vector4 Math Header

#ifndef VECTOR4_H
#define VECTOR4_H

#include "utils.h"

typedef __attribute__((aligned(16))) struct Vector4 {
    float x;
    float y;
    float z;
    float w;
} Vector4;

#include "math/matrix4.h"

void vector4_add_vector4(Vector4* vector, Vector4* rhs);

void vector4_mul_matrix4(Vector4* vector, Matrix4* rhs);

#endif
