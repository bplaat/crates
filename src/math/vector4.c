// PlaatCraft - Vector4 Math

#include "math/vector4.h"
#ifndef NO_SSIMD
    #include <emmintrin.h>
#endif

void vector4_add(Vector4* vector, Vector4* rhs) {
    #ifndef NO_SSIMD
        _mm_store_ps(&vector->x, _mm_add_ps(_mm_load_ps(&vector->x), _mm_load_ps(&rhs->x)));
    #else
        vector->x += rhs->x;
        vector->y += rhs->y;
        vector->z += rhs->z;
        vector->w += rhs->w;
    #endif
}

void vector4_mul(Vector4* vector, Matrix4* rhs) {
    #ifndef NO_SSIMD
        __m128 row1 = _mm_mul_ps(_mm_load_ps(&rhs->m11), _mm_load1_ps(&vector->x));
        __m128 row2 = _mm_mul_ps(_mm_load_ps(&rhs->m21), _mm_load1_ps(&vector->y));
        __m128 row3 = _mm_mul_ps(_mm_load_ps(&rhs->m31), _mm_load1_ps(&vector->z));
        __m128 row4 = _mm_mul_ps(_mm_load_ps(&rhs->m41), _mm_load1_ps(&vector->w));
        _mm_store_ps(&vector->x, _mm_add_ps(
            _mm_add_ps(row1, row2),
            _mm_add_ps(row3, row4)
        ));
    #else
        float _x = rhs.m11 * x + rhs.m21 * y + rhs.m31 * z + rhs.m41 * w;
        float _y = rhs.m12 * x + rhs.m22 * y + rhs.m32 * z + rhs.m42 * w;
        float _z = rhs.m13 * x + rhs.m23 * y + rhs.m33 * z + rhs.m43 * w;
        float _w = rhs.m14 * x + rhs.m24 * y + rhs.m34 * z + rhs.m44 * w;

        x = _x;
        y = _y;
        z = _z;
        w = _w;
    #endif
}
