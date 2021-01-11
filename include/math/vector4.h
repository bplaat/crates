// PlaatCraft - Vector4 Math Header

#ifndef VECTOR4_H
#define VECTOR4_H

#include "utils.h"

typedef struct _MM_ALIGN16 Vector4 {
    float x;
    float y;
    float z;
    float w;
} Vector4;

#include "math/matrix4.h"

void vector4_add(Vector4* vector, Vector4* rhs);

void vector4_mul(Vector4* vector, Matrix4* rhs);

#endif
