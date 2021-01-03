// PlaatCraft - Vector4

#include "vector4.h"
#include <emmintrin.h>

void vector4_add(Vector4* vector, Vector4* rhs) {
    _mm_store_ps(&vector->x, _mm_add_ps(_mm_load_ps(&vector->x), _mm_load_ps(&rhs->x)));
}

void vector4_mul(Vector4* vector, Matrix4* rhs) {
    __m128 row1 = _mm_mul_ps(_mm_load_ps(&rhs->m11), _mm_load1_ps(&vector->x));
    __m128 row2 = _mm_mul_ps(_mm_load_ps(&rhs->m21), _mm_load1_ps(&vector->y));
    __m128 row3 = _mm_mul_ps(_mm_load_ps(&rhs->m31), _mm_load1_ps(&vector->z));
    __m128 row4 = _mm_mul_ps(_mm_load_ps(&rhs->m41), _mm_load1_ps(&vector->w));
    _mm_store_ps(&vector->x, _mm_add_ps(
        _mm_add_ps(row1, row2),
        _mm_add_ps(row3, row4)
    ));
}
