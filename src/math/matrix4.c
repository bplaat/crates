// PlaatCraft - Matrix4 Math

#include "math/matrix4.h"
#ifndef NO_SSIMD
    #include <emmintrin.h>
#endif
#define _USE_MATH_DEFINES
#include <math.h>

void matrix4_identity(Matrix4* matrix) {
    #ifndef NO_SIMD
        _mm_store_ps(&matrix->m11, _mm_setr_ps(1, 0, 0, 0));
        _mm_store_ps(&matrix->m21, _mm_setr_ps(0, 1, 0, 0));
        _mm_store_ps(&matrix->m31, _mm_setr_ps(0, 0, 1, 0));
        _mm_store_ps(&matrix->m41, _mm_setr_ps(0, 0, 0, 1));
    #else
        matrix->m11 = 1; matrix->m12 = 0; matrix->m13 = 0; matrix->m14 = 0;
        matrix->m21 = 0; matrix->m22 = 1; matrix->m23 = 0; matrix->m24 = 0;
        matrix->m31 = 0; matrix->m32 = 0; matrix->m33 = 1; matrix->m34 = 0;
        matrix->m41 = 0; matrix->m42 = 0; matrix->m43 = 0; matrix->m44 = 1;
    #endif
}

void matrix4_perspective(Matrix4* matrix, float fov, float aspect, float near, float far) {
    float f = tan(M_PI * 0.5 - 0.5 * fov);
    float r = 1 / (near - far);

    #ifndef NO_SIMD
        _mm_store_ps(&matrix->m11, _mm_setr_ps(f / aspect, 0, 0, 0));
        _mm_store_ps(&matrix->m21, _mm_setr_ps(0, f, 0, 0));
        _mm_store_ps(&matrix->m31, _mm_setr_ps(0, 0, (near + far) * r, -1));
        _mm_store_ps(&matrix->m41, _mm_setr_ps(0, 0, near * far * r * 2, 0));
    #else
        matrix->m11 = f / aspect; matrix->m12 = 0; matrix->m13 = 0; matrix->m14 = 0;
        matrix->m21 = 0; matrix->m22 = f; matrix->m23 = 0; matrix->m24 = 0;
        matrix->m31 = 0; matrix->m32 = 0; matrix->m33 = (near + far) * r; matrix->m34 = -1;
        matrix->m41 = 0; matrix->m42 = 0; matrix->m43 = near * far * r * 2; matrix->m44 = 0;
    #endif
}

void matrix4_translate(Matrix4* matrix, Vector4* position) {
    #ifndef NO_SIMD
        _mm_store_ps(&matrix->m11, _mm_setr_ps(1, 0, 0, 0));
        _mm_store_ps(&matrix->m21, _mm_setr_ps(0, 1, 0, 0));
        _mm_store_ps(&matrix->m31, _mm_setr_ps(0, 0, 1, 0));
        _mm_store_ps(&matrix->m41, _mm_setr_ps(position->x, position->y, position->z, 1));
    #else
        matrix->m11 = 1; matrix->m12 = 0; matrix->m13 = 0; matrix->m14 = 0;
        matrix->m21 = 0; matrix->m22 = 1; matrix->m23 = 0; matrix->m24 = 0;
        matrix->m31 = 0; matrix->m32 = 0; matrix->m33 = 1; matrix->m34 = 0;
        matrix->m41 = position->x; matrix->m42 = position->y; matrix->m43 = position->z; matrix->m44 = 1;
    #endif
}

void matrix4_rotate_x(Matrix4* matrix, float x) {
    float c = cos(x);
    float s = sin(x);

    #ifndef NO_SIMD
        _mm_store_ps(&matrix->m11, _mm_setr_ps(1, 0, 0, 0));
        _mm_store_ps(&matrix->m21, _mm_setr_ps(0, c, s, 0));
        _mm_store_ps(&matrix->m31, _mm_setr_ps(0, -s, c, 0));
        _mm_store_ps(&matrix->m41, _mm_setr_ps(0, 0, 0, 1));
    #else
        matrix->m11 = 1; matrix->m12 = 0; matrix->m13 = 0; matrix->m14 = 0;
        matrix->m21 = 0; matrix->m22 = c; matrix->m23 = s; matrix->m24 = 0;
        matrix->m31 = 0; matrix->m32 = -s; matrix->m33 = c; matrix->m34 = 0;
        matrix->m41 = 0; matrix->m42 = 0; matrix->m43 = 0; matrix->m44 = 1;
    #endif
}

void matrix4_rotate_y(Matrix4* matrix, float y) {
    float c = cos(y);
    float s = sin(y);

    #ifndef NO_SIMD
        _mm_store_ps(&matrix->m11, _mm_setr_ps(c, 0, -s, 0));
        _mm_store_ps(&matrix->m21, _mm_setr_ps(0, 1, 0, 0));
        _mm_store_ps(&matrix->m31, _mm_setr_ps(s, 0, c, 0));
        _mm_store_ps(&matrix->m41, _mm_setr_ps(0, 0, 0, 1));
    #else
        matrix->m11 = c; matrix->m12 = 0; matrix->m13 = -s; matrix->m14 = 0;
        matrix->m21 = 0; matrix->m22 = 1; matrix->m23 = 0; matrix->m24 = 0;
        matrix->m31 = s; matrix->m32 = 0; matrix->m33 = c; matrix->m34 = 0;
        matrix->m41 = 0; matrix->m42 = 0; matrix->m43 = 0; matrix->m44 = 1;
    #endif
}

void matrix4_rotate_z(Matrix4* matrix, float z) {
    float c = cos(z);
    float s = sin(z);

    #ifndef NO_SIMD
        _mm_store_ps(&matrix->m11, _mm_setr_ps(c, s, 0, 0));
        _mm_store_ps(&matrix->m21, _mm_setr_ps(-s, c, 0, 0));
        _mm_store_ps(&matrix->m31, _mm_setr_ps(0, 0, 1, 0));
        _mm_store_ps(&matrix->m41, _mm_setr_ps(0, 0, 0, 1));
    #else
        matrix->m11 = c; matrix->m12 = s; matrix->m13 = 0; matrix->m14 = 0;
        matrix->m21 = -s; matrix->m22 = c; matrix->m23 = 0; matrix->m24 = 0;
        matrix->m31 = 0; matrix->m32 = 0; matrix->m33 = 1; matrix->m34 = 0;
        matrix->m41 = 0; matrix->m42 = 0; matrix->m43 = 0; matrix->m44 = 1;
    #endif
}

void matrix4_scale(Matrix4* matrix, Vector4* scale) {
    #ifndef NO_SIMD
        _mm_store_ps(&matrix->m11, _mm_setr_ps(scale->x, 0, 0, 0));
        _mm_store_ps(&matrix->m21, _mm_setr_ps(0, scale->y, 0, 0));
        _mm_store_ps(&matrix->m31, _mm_setr_ps(0, 0, scale->z, 0));
        _mm_store_ps(&matrix->m41, _mm_setr_ps(0, 0, 0, 1));
    #else
        matrix->m11 = scale->x; matrix->m12 = 0; matrix->m13 = 0; matrix->m14 = 0;
        matrix->m21 = 0; matrix->m22 = scale->y; matrix->m23 = 0; matrix->m24 = 0;
        matrix->m31 = 0; matrix->m32 = 0; matrix->m33 = scale->z; matrix->m34 = 0;
        matrix->m41 = 0; matrix->m42 = 0; matrix->m43 = 0; matrix->m44 = 1;
    #endif
}

void matrix4_mul(Matrix4* matrix, Matrix4* rhs) {
    #ifndef NO_SIMD
        __m128 row1 = _mm_load_ps(&matrix->m11);
        __m128 row2 = _mm_load_ps(&matrix->m21);
        __m128 row3 = _mm_load_ps(&matrix->m31);
        __m128 row4 = _mm_load_ps(&matrix->m41);

        for (int y = 0; y < 4; y++) {
            __m128 col1 = _mm_load1_ps(&rhs->m11 + y * 4 + 0);
            __m128 col2 = _mm_load1_ps(&rhs->m11 + y * 4 + 1);
            __m128 col3 = _mm_load1_ps(&rhs->m11 + y * 4 + 2);
            __m128 col4 = _mm_load1_ps(&rhs->m11 + y * 4 + 3);

            _mm_store_ps(&matrix->m11 + y * 4, _mm_add_ps(
                _mm_add_ps(
                    _mm_mul_ps(col1, row1),
                    _mm_mul_ps(col2, row2)
                ),
                _mm_add_ps(
                    _mm_mul_ps(col3, row3),
                    _mm_mul_ps(col4, row4)
                )
            ));
        }
    #else
        float _m11 = rhs.m11 * m11 + rhs.m12 * m21 + rhs.m13 * m31 + rhs.m14 * m41;
        float _m12 = rhs.m11 * m12 + rhs.m12 * m22 + rhs.m13 * m32 + rhs.m14 * m42;
        float _m13 = rhs.m11 * m13 + rhs.m12 * m23 + rhs.m13 * m33 + rhs.m14 * m43;
        float _m14 = rhs.m11 * m14 + rhs.m12 * m24 + rhs.m13 * m34 + rhs.m14 * m44;

        float _m21 = rhs.m21 * m11 + rhs.m22 * m21 + rhs.m23 * m31 + rhs.m24 * m41;
        float _m22 = rhs.m21 * m12 + rhs.m22 * m22 + rhs.m23 * m32 + rhs.m24 * m42;
        float _m23 = rhs.m21 * m13 + rhs.m22 * m23 + rhs.m23 * m33 + rhs.m24 * m43;
        float _m24 = rhs.m21 * m14 + rhs.m22 * m24 + rhs.m23 * m34 + rhs.m24 * m44;

        float _m31 = rhs.m31 * m11 + rhs.m32 * m21 + rhs.m33 * m31 + rhs.m34 * m41;
        float _m32 = rhs.m31 * m12 + rhs.m32 * m22 + rhs.m33 * m32 + rhs.m34 * m42;
        float _m33 = rhs.m31 * m13 + rhs.m32 * m23 + rhs.m33 * m33 + rhs.m34 * m43;
        float _m34 = rhs.m31 * m14 + rhs.m32 * m24 + rhs.m33 * m34 + rhs.m34 * m44;

        float _m41 = rhs.m41 * m11 + rhs.m42 * m21 + rhs.m43 * m31 + rhs.m44 * m41;
        float _m42 = rhs.m41 * m12 + rhs.m42 * m22 + rhs.m43 * m32 + rhs.m44 * m42;
        float _m43 = rhs.m41 * m13 + rhs.m42 * m23 + rhs.m43 * m33 + rhs.m44 * m43;
        float _m44 = rhs.m41 * m14 + rhs.m42 * m24 + rhs.m43 * m34 + rhs.m44 * m44;

        m11 = _m11; m12 = _m12; m13 = _m13; m14 = _m14;
        m21 = _m21; m22 = _m22; m23 = _m23; m24 = _m24;
        m31 = _m31; m32 = _m32; m33 = _m33; m34 = _m34;
        m41 = _m41; m42 = _m42; m43 = _m43; m44 = _m44;
    #endif
}
