// PlaatCraft - Matrix4

#include "matrix4.h"
#include <emmintrin.h>
#define _USE_MATH_DEFINES
#include <math.h>

void matrix4_identity(Matrix4* matrix) {
    _mm_store_ps(&matrix->m11, _mm_setr_ps(1, 0, 0, 0));
    _mm_store_ps(&matrix->m21, _mm_setr_ps(0, 1, 0, 0));
    _mm_store_ps(&matrix->m31, _mm_setr_ps(0, 0, 1, 0));
    _mm_store_ps(&matrix->m41, _mm_setr_ps(0, 0, 0, 1));
}

void matrix4_perspective(Matrix4* matrix, float fov, float aspect, float near, float far) {
    float f = tan(M_PI * 0.5 - 0.5 * fov);
    float r = 1 / (near - far);
    _mm_store_ps(&matrix->m11, _mm_setr_ps(f / aspect, 0, 0, 0));
    _mm_store_ps(&matrix->m21, _mm_setr_ps(0, f, 0, 0));
    _mm_store_ps(&matrix->m31, _mm_setr_ps(0, 0, (near + far) * r, -1));
    _mm_store_ps(&matrix->m41, _mm_setr_ps(0, 0, near * far * r * 2, 0));
}

void matrix4_translate(Matrix4* matrix, Vector4* position) {
    _mm_store_ps(&matrix->m11, _mm_setr_ps(1, 0, 0, 0));
    _mm_store_ps(&matrix->m21, _mm_setr_ps(0, 1, 0, 0));
    _mm_store_ps(&matrix->m31, _mm_setr_ps(0, 0, 1, 0));
    _mm_store_ps(&matrix->m41, _mm_setr_ps(position->x, position->y, position->z, 1));
}

void matrix4_rotate_x(Matrix4* matrix, float x) {
    float c = cos(x);
    float s = sin(x);
    _mm_store_ps(&matrix->m11, _mm_setr_ps(1, 0, 0, 0));
    _mm_store_ps(&matrix->m21, _mm_setr_ps(0, c, s, 0));
    _mm_store_ps(&matrix->m31, _mm_setr_ps(0, -s, c, 0));
    _mm_store_ps(&matrix->m41, _mm_setr_ps(0, 0, 0, 1));
}

void matrix4_rotate_y(Matrix4* matrix, float y) {
    float c = cos(y);
    float s = sin(y);
    _mm_store_ps(&matrix->m11, _mm_setr_ps(c, 0, -s, 0));
    _mm_store_ps(&matrix->m21, _mm_setr_ps(0, 1, 0, 0));
    _mm_store_ps(&matrix->m31, _mm_setr_ps(s, 0, c, 0));
    _mm_store_ps(&matrix->m41, _mm_setr_ps(0, 0, 0, 1));
}

void matrix4_rotate_z(Matrix4* matrix, float z) {
    float c = cos(z);
    float s = sin(z);
    _mm_store_ps(&matrix->m11, _mm_setr_ps(c, s, 0, 0));
    _mm_store_ps(&matrix->m21, _mm_setr_ps(-s, c, 0, 0));
    _mm_store_ps(&matrix->m31, _mm_setr_ps(0, 0, 1, 0));
    _mm_store_ps(&matrix->m41, _mm_setr_ps(0, 0, 0, 1));
}

void matrix4_scale(Matrix4* matrix, Vector4* scale) {
    _mm_store_ps(&matrix->m11, _mm_setr_ps(scale->x, 0, 0, 0));
    _mm_store_ps(&matrix->m21, _mm_setr_ps(0, scale->y, 0, 0));
    _mm_store_ps(&matrix->m31, _mm_setr_ps(0, 0, scale->z, 0));
    _mm_store_ps(&matrix->m41, _mm_setr_ps(0, 0, 0, 1));
}

void matrix4_mul(Matrix4* matrix, Matrix4* rhs) {
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
}
