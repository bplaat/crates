// PlaatCraft - Vector4 Math

#include "math/vector4.h"
#ifndef NO_SIMD
    #include <emmintrin.h>
#endif

void vector4_add(Vector4* vector, Vector4* rhs) {
    #ifndef NO_SIMD
        _mm_store_ps(&vector->x, _mm_add_ps(_mm_load_ps(&vector->x), _mm_load_ps(&rhs->x)));
    #else
        vector->x += rhs->x;
        vector->y += rhs->y;
        vector->z += rhs->z;
        vector->w += rhs->w;
    #endif
}

void vector4_mul(Vector4* vector, Matrix4* rhs) {
    #ifndef NO_SIMD
        __m128 row1 = _mm_mul_ps(_mm_load_ps(&rhs->m11), _mm_load1_ps(&vector->x));
        __m128 row2 = _mm_mul_ps(_mm_load_ps(&rhs->m21), _mm_load1_ps(&vector->y));
        __m128 row3 = _mm_mul_ps(_mm_load_ps(&rhs->m31), _mm_load1_ps(&vector->z));
        __m128 row4 = _mm_mul_ps(_mm_load_ps(&rhs->m41), _mm_load1_ps(&vector->w));
        _mm_store_ps(&vector->x, _mm_add_ps(
            _mm_add_ps(row1, row2),
            _mm_add_ps(row3, row4)
        ));
    #else
        float _x = rhs->m11 * vector->x + rhs->m21 * vector->y + rhs->m31 * vector->z + rhs->m41 * vector->w;
        float _y = rhs->m12 * vector->x + rhs->m22 * vector->y + rhs->m32 * vector->z + rhs->m42 * vector->w;
        float _z = rhs->m13 * vector->x + rhs->m23 * vector->y + rhs->m33 * vector->z + rhs->m43 * vector->w;
        float _w = rhs->m14 * vector->x + rhs->m24 * vector->y + rhs->m34 * vector->z + rhs->m44 * vector->w;

        vector->x = _x;
        vector->y = _y;
        vector->z = _z;
        vector->w = _w;
    #endif
}
