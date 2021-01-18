// PlaatCraft - Vector4 Math Header

#ifndef VECTOR4_H
#define VECTOR4_H

#include "utils.h"

#ifndef NO_SIMD
typedef struct _MM_ALIGN16 Vector4 {
#else
typedef struct Vector4 {
#endif
    float x;
    float y;
    float z;
    float w;
} Vector4;

#include "math/matrix4.h"

void vector4_add(Vector4* vector, Vector4* rhs);

void vector4_mul(Vector4* vector, Matrix4* rhs);

#endif
