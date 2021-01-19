// PlaatCraft - Matrix4 Math

#include "math/matrix4.h"
#ifndef NO_SIMD
    #include <emmintrin.h>
#endif
#include "utils.h"
#include "log.h"

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

void matrix4_flat_projection(Matrix4* matrix, int viewport_width, int viewport_height) {
    #ifndef NO_SIMD
        _mm_store_ps(&matrix->m11, _mm_setr_ps(2 / (float)viewport_width, 0, 0, 0));
        _mm_store_ps(&matrix->m21, _mm_setr_ps(0, -2 / (float)viewport_height, 0, 0));
        _mm_store_ps(&matrix->m31, _mm_setr_ps(0, 0, 1, 0));
        _mm_store_ps(&matrix->m41, _mm_setr_ps(-1, 1, 0, 1));
    #else
        matrix->m11 = 2 / (float)viewport_width; matrix->m12 = 0; matrix->m13 = 0; matrix->m14 = 0;
        matrix->m21 = 0; matrix->m22 = -2 / (float)viewport_height; matrix->m23 = 0; matrix->m24 = 0;
        matrix->m31 = 0; matrix->m32 = 0; matrix->m33 = 1; matrix->m34 = 0;
        matrix->m41 = -1; matrix->m42 = 1; matrix->m43 = 0; matrix->m44 = 1;
    #endif
}

void matrix4_flat_rect(Matrix4* matrix, int x, int y, int width, int height) {
    Vector4 translate_vector = { x + width / 2, y + height / 2, 0, 1 };
    matrix4_translate(matrix, &translate_vector);

    Matrix4 temp_matrix;
    Vector4 scale_vector = { width, height, 0, 1 };
    matrix4_scale(&temp_matrix, &scale_vector);
    matrix4_mul(matrix, &temp_matrix);
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
        float _m11 = rhs->m11 * matrix->m11 + rhs->m12 * matrix->m21 + rhs->m13 * matrix->m31 + rhs->m14 * matrix->m41;
        float _m12 = rhs->m11 * matrix->m12 + rhs->m12 * matrix->m22 + rhs->m13 * matrix->m32 + rhs->m14 * matrix->m42;
        float _m13 = rhs->m11 * matrix->m13 + rhs->m12 * matrix->m23 + rhs->m13 * matrix->m33 + rhs->m14 * matrix->m43;
        float _m14 = rhs->m11 * matrix->m14 + rhs->m12 * matrix->m24 + rhs->m13 * matrix->m34 + rhs->m14 * matrix->m44;

        float _m21 = rhs->m21 * matrix->m11 + rhs->m22 * matrix->m21 + rhs->m23 * matrix->m31 + rhs->m24 * matrix->m41;
        float _m22 = rhs->m21 * matrix->m12 + rhs->m22 * matrix->m22 + rhs->m23 * matrix->m32 + rhs->m24 * matrix->m42;
        float _m23 = rhs->m21 * matrix->m13 + rhs->m22 * matrix->m23 + rhs->m23 * matrix->m33 + rhs->m24 * matrix->m43;
        float _m24 = rhs->m21 * matrix->m14 + rhs->m22 * matrix->m24 + rhs->m23 * matrix->m34 + rhs->m24 * matrix->m44;

        float _m31 = rhs->m31 * matrix->m11 + rhs->m32 * matrix->m21 + rhs->m33 * matrix->m31 + rhs->m34 * matrix->m41;
        float _m32 = rhs->m31 * matrix->m12 + rhs->m32 * matrix->m22 + rhs->m33 * matrix->m32 + rhs->m34 * matrix->m42;
        float _m33 = rhs->m31 * matrix->m13 + rhs->m32 * matrix->m23 + rhs->m33 * matrix->m33 + rhs->m34 * matrix->m43;
        float _m34 = rhs->m31 * matrix->m14 + rhs->m32 * matrix->m24 + rhs->m33 * matrix->m34 + rhs->m34 * matrix->m44;

        float _m41 = rhs->m41 * matrix->m11 + rhs->m42 * matrix->m21 + rhs->m43 * matrix->m31 + rhs->m44 * matrix->m41;
        float _m42 = rhs->m41 * matrix->m12 + rhs->m42 * matrix->m22 + rhs->m43 * matrix->m32 + rhs->m44 * matrix->m42;
        float _m43 = rhs->m41 * matrix->m13 + rhs->m42 * matrix->m23 + rhs->m43 * matrix->m33 + rhs->m44 * matrix->m43;
        float _m44 = rhs->m41 * matrix->m14 + rhs->m42 * matrix->m24 + rhs->m43 * matrix->m34 + rhs->m44 * matrix->m44;

        matrix->m11 = _m11; matrix->m12 = _m12; matrix->m13 = _m13; matrix->m14 = _m14;
        matrix->m21 = _m21; matrix->m22 = _m22; matrix->m23 = _m23; matrix->m24 = _m24;
        matrix->m31 = _m31; matrix->m32 = _m32; matrix->m33 = _m33; matrix->m34 = _m34;
        matrix->m41 = _m41; matrix->m42 = _m42; matrix->m43 = _m43; matrix->m44 = _m44;
    #endif
}


void matrix4_transpose(Matrix4* matrix) {
    #ifndef NO_SIMD
        __m128 row1 = _mm_load_ps(&matrix->m11);
        __m128 row2 = _mm_load_ps(&matrix->m21);
        __m128 row3 = _mm_load_ps(&matrix->m31);
        __m128 row4 = _mm_load_ps(&matrix->m41);
        _MM_TRANSPOSE4_PS(row1, row2, row3, row4);
        _mm_store_ps(&matrix->m11, row1);
        _mm_store_ps(&matrix->m21, row2);
        _mm_store_ps(&matrix->m31, row3);
        _mm_store_ps(&matrix->m41, row4);
    #else
        float _m12 = matrix->m21;
        float _m13 = matrix->m31;
        float _m14 = matrix->m41;

        float _m21 = matrix->m12;
        float _m23 = matrix->m32;
        float _m24 = matrix->m42;

        float _m31 = matrix->m13;
        float _m32 = matrix->m23;
        float _m34 = matrix->m43;

        float _m41 = matrix->m14;
        float _m42 = matrix->m24;
        float _m43 = matrix->m34;

        matrix->m12 = _m12; matrix->m13 = _m13; matrix->m14 = _m14;
        matrix->m21 = _m21; matrix->m23 = _m23; matrix->m24 = _m24;
        matrix->m31 = _m31; matrix->m32 = _m32; matrix->m34 = _m34;
        matrix->m41 = _m41; matrix->m42 = _m42; matrix->m43 = _m43;
    #endif
}

void matrix4_inverse(Matrix4* matrix) {
    double inv[16], det;
    int i;

    float *m = &matrix->m11;

    inv[0] = m[5]  * m[10] * m[15] -
             m[5]  * m[11] * m[14] -
             m[9]  * m[6]  * m[15] +
             m[9]  * m[7]  * m[14] +
             m[13] * m[6]  * m[11] -
             m[13] * m[7]  * m[10];

    inv[4] = -m[4]  * m[10] * m[15] +
              m[4]  * m[11] * m[14] +
              m[8]  * m[6]  * m[15] -
              m[8]  * m[7]  * m[14] -
              m[12] * m[6]  * m[11] +
              m[12] * m[7]  * m[10];

    inv[8] = m[4]  * m[9] * m[15] -
             m[4]  * m[11] * m[13] -
             m[8]  * m[5] * m[15] +
             m[8]  * m[7] * m[13] +
             m[12] * m[5] * m[11] -
             m[12] * m[7] * m[9];

    inv[12] = -m[4]  * m[9] * m[14] +
               m[4]  * m[10] * m[13] +
               m[8]  * m[5] * m[14] -
               m[8]  * m[6] * m[13] -
               m[12] * m[5] * m[10] +
               m[12] * m[6] * m[9];

    inv[1] = -m[1]  * m[10] * m[15] +
              m[1]  * m[11] * m[14] +
              m[9]  * m[2] * m[15] -
              m[9]  * m[3] * m[14] -
              m[13] * m[2] * m[11] +
              m[13] * m[3] * m[10];

    inv[5] = m[0]  * m[10] * m[15] -
             m[0]  * m[11] * m[14] -
             m[8]  * m[2] * m[15] +
             m[8]  * m[3] * m[14] +
             m[12] * m[2] * m[11] -
             m[12] * m[3] * m[10];

    inv[9] = -m[0]  * m[9] * m[15] +
              m[0]  * m[11] * m[13] +
              m[8]  * m[1] * m[15] -
              m[8]  * m[3] * m[13] -
              m[12] * m[1] * m[11] +
              m[12] * m[3] * m[9];

    inv[13] = m[0]  * m[9] * m[14] -
              m[0]  * m[10] * m[13] -
              m[8]  * m[1] * m[14] +
              m[8]  * m[2] * m[13] +
              m[12] * m[1] * m[10] -
              m[12] * m[2] * m[9];

    inv[2] = m[1]  * m[6] * m[15] -
             m[1]  * m[7] * m[14] -
             m[5]  * m[2] * m[15] +
             m[5]  * m[3] * m[14] +
             m[13] * m[2] * m[7] -
             m[13] * m[3] * m[6];

    inv[6] = -m[0]  * m[6] * m[15] +
              m[0]  * m[7] * m[14] +
              m[4]  * m[2] * m[15] -
              m[4]  * m[3] * m[14] -
              m[12] * m[2] * m[7] +
              m[12] * m[3] * m[6];

    inv[10] = m[0]  * m[5] * m[15] -
              m[0]  * m[7] * m[13] -
              m[4]  * m[1] * m[15] +
              m[4]  * m[3] * m[13] +
              m[12] * m[1] * m[7] -
              m[12] * m[3] * m[5];

    inv[14] = -m[0]  * m[5] * m[14] +
               m[0]  * m[6] * m[13] +
               m[4]  * m[1] * m[14] -
               m[4]  * m[2] * m[13] -
               m[12] * m[1] * m[6] +
               m[12] * m[2] * m[5];

    inv[3] = -m[1] * m[6] * m[11] +
              m[1] * m[7] * m[10] +
              m[5] * m[2] * m[11] -
              m[5] * m[3] * m[10] -
              m[9] * m[2] * m[7] +
              m[9] * m[3] * m[6];

    inv[7] = m[0] * m[6] * m[11] -
             m[0] * m[7] * m[10] -
             m[4] * m[2] * m[11] +
             m[4] * m[3] * m[10] +
             m[8] * m[2] * m[7] -
             m[8] * m[3] * m[6];

    inv[11] = -m[0] * m[5] * m[11] +
               m[0] * m[7] * m[9] +
               m[4] * m[1] * m[11] -
               m[4] * m[3] * m[9] -
               m[8] * m[1] * m[7] +
               m[8] * m[3] * m[5];

    inv[15] = m[0] * m[5] * m[10] -
              m[0] * m[6] * m[9] -
              m[4] * m[1] * m[10] +
              m[4] * m[2] * m[9] +
              m[8] * m[1] * m[6] -
              m[8] * m[2] * m[5];

    det = m[0] * inv[0] + m[1] * inv[4] + m[2] * inv[8] + m[3] * inv[12];

    if (det == 0) {
        log_error("Inverse matrix determinant is zero!");
    }

    det = 1.0 / det;

    for (i = 0; i < 16; i++)
        m[i] = m[i] * det;
}
