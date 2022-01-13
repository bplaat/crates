// PlaatCraft - Matrix4 Math

#include "math/matrix4.h"
#ifdef ENABLE_NEON_SIMD
    #include <arm_neon.h>
#endif
#ifdef ENABLE_SSE2_SIMD
    #include <emmintrin.h>
#endif
#include "utils.h"
#include "log.h"

void matrix4_identity(Matrix4 *matrix) {
    matrix->elements[0] = 1; matrix->elements[1] = 0; matrix->elements[2] = 0; matrix->elements[3] = 0;
    matrix->elements[4] = 0; matrix->elements[5] = 1; matrix->elements[6] = 0; matrix->elements[7] = 0;
    matrix->elements[8] = 0; matrix->elements[9] = 0; matrix->elements[10] = 1; matrix->elements[11] = 0;
    matrix->elements[12] = 0; matrix->elements[13] = 0; matrix->elements[14] = 0; matrix->elements[15] = 1;
}

void matrix4_perspective(Matrix4 *matrix, float fov, float aspect, float near, float far) {
    float f = tanf(M_PI * 0.5f - 0.5f * fov);
    float r = 1.0 / (near - far);
    matrix->elements[0] = f / aspect; matrix->elements[1] = 0; matrix->elements[2] = 0; matrix->elements[3] = 0;
    matrix->elements[4] = 0; matrix->elements[5] = f; matrix->elements[6] = 0; matrix->elements[7] = 0;
    matrix->elements[8] = 0; matrix->elements[9] = 0; matrix->elements[10] = (near + far) * r; matrix->elements[11] = -1;
    matrix->elements[12] = 0; matrix->elements[13] = 0; matrix->elements[14] = near * far * r * 2; matrix->elements[15] = 0;
}

void matrix4_translate(Matrix4 *matrix, Vector4* position) {
    matrix->elements[0] = 1; matrix->elements[1] = 0; matrix->elements[2] = 0; matrix->elements[3] = 0;
    matrix->elements[4] = 0; matrix->elements[5] = 1; matrix->elements[6] = 0; matrix->elements[7] = 0;
    matrix->elements[8] = 0; matrix->elements[9] = 0; matrix->elements[10] = 1; matrix->elements[11] = 0;
    matrix->elements[12] = position->x; matrix->elements[13] = position->y; matrix->elements[14] = position->z; matrix->elements[15] = 1;
}

void matrix4_rotate_x(Matrix4 *matrix, float x) {
    float s = sinf(x);
    float c = cosf(x);
    matrix->elements[0] = 1; matrix->elements[1] = 0; matrix->elements[2] = 0; matrix->elements[3] = 0;
    matrix->elements[4] = 0; matrix->elements[5] = c; matrix->elements[6] = s; matrix->elements[7] = 0;
    matrix->elements[8] = 0; matrix->elements[9] = -s; matrix->elements[10] = c; matrix->elements[11] = 0;
    matrix->elements[12] = 0; matrix->elements[13] = 0; matrix->elements[14] = 0; matrix->elements[15] = 1;
}

void matrix4_rotate_y(Matrix4 *matrix, float y) {
    float s = sinf(y);
    float c = cosf(y);
    matrix->elements[0] = c; matrix->elements[1] = 0; matrix->elements[2] = -s; matrix->elements[3] = 0;
    matrix->elements[4] = 0; matrix->elements[5] = 1; matrix->elements[6] = 0; matrix->elements[7] = 0;
    matrix->elements[8] = s; matrix->elements[9] = 0; matrix->elements[10] = c; matrix->elements[11] = 0;
    matrix->elements[12] = 0; matrix->elements[13] = 0; matrix->elements[14] = 0; matrix->elements[15] = 1;
}

void matrix4_rotate_z(Matrix4 *matrix, float z) {
    float s = sinf(z);
    float c = cosf(z);
    matrix->elements[0] = c; matrix->elements[1] = s; matrix->elements[2] = 0; matrix->elements[3] = 0;
    matrix->elements[4] = -s; matrix->elements[5] = c; matrix->elements[6] = 0; matrix->elements[7] = 0;
    matrix->elements[8] = 0; matrix->elements[9] = 0; matrix->elements[10] = 1; matrix->elements[11] = 0;
    matrix->elements[12] = 0; matrix->elements[13] = 0; matrix->elements[14] = 0; matrix->elements[15] = 1;
}

void matrix4_scale(Matrix4 *matrix, Vector4* scale) {
    matrix->elements[0] = scale->x; matrix->elements[1] = 0; matrix->elements[2] = 0; matrix->elements[3] = 0;
    matrix->elements[4] = 0; matrix->elements[5] = scale->y; matrix->elements[6] = 0; matrix->elements[7] = 0;
    matrix->elements[8] = 0; matrix->elements[9] = 0; matrix->elements[10] = scale->z; matrix->elements[11] = 0;
    matrix->elements[12] = 0; matrix->elements[13] = 0; matrix->elements[14] = 0; matrix->elements[15] = 1;
}


void matrix4_flat_projection(Matrix4* matrix, int viewport_width, int viewport_height) {
    matrix->elements[0] = 2 / (float)viewport_width; matrix->elements[1] = 0; matrix->elements[2] = 0; matrix->elements[3] = 0;
    matrix->elements[4] = 0; matrix->elements[5] = -2 / (float)viewport_height; matrix->elements[6] = 0; matrix->elements[7] = 0;
    matrix->elements[8] = 0; matrix->elements[9] = 0; matrix->elements[10] = 1; matrix->elements[11] = 0;
    matrix->elements[12] = -1; matrix->elements[13] = 1; matrix->elements[14] = 0; matrix->elements[15] = 1;
}

void matrix4_flat_rect(Matrix4* matrix, int x, int y, int width, int height) {
    Vector4 translate_vector = { x + width / 2, y + height / 2, 0, 1 };
    matrix4_translate(matrix, &translate_vector);

    Matrix4 temp_matrix;
    Vector4 scale_vector = { width, height, 0, 1 };
    matrix4_scale(&temp_matrix, &scale_vector);
    matrix4_mul_matrix4(matrix, &temp_matrix);
}

void matrix4_mul_matrix4(Matrix4* matrix, Matrix4* rhs) {
    #ifdef ENABLE_NEON_SIMD
        float32x4_t a0 = vld1q_f32(&matrix->elements[0]);
        float32x4_t a1 = vld1q_f32(&matrix->elements[4]);
        float32x4_t a2 = vld1q_f32(&matrix->elements[8]);
        float32x4_t a3 = vld1q_f32(&matrix->elements[12]);

        float32x4_t b = vld1q_f32(&rhs->elements[0]);
        float32x4_t sum = vmulq_f32(a0, vmovq_n_f32(b[0]));
        sum = vmlaq_f32(sum, a1, vmovq_n_f32(b[1]));
        sum = vmlaq_f32(sum, a2, vmovq_n_f32(b[2]));
        sum = vmlaq_f32(sum, a3, vmovq_n_f32(b[3]));
        vst1q_f32(&matrix->elements[0], sum);

        b = vld1q_f32(&rhs->elements[4]);
        sum = vmulq_f32(a0, vmovq_n_f32(b[0]));
        sum = vmlaq_f32(sum, a1, vmovq_n_f32(b[1]));
        sum = vmlaq_f32(sum, a2, vmovq_n_f32(b[2]));
        sum = vmlaq_f32(sum, a3, vmovq_n_f32(b[3]));
        vst1q_f32(&matrix->elements[4], sum);

        b = vld1q_f32(&rhs->elements[8]);
        sum = vmulq_f32(a0, vmovq_n_f32(b[0]));
        sum = vmlaq_f32(sum, a1, vmovq_n_f32(b[1]));
        sum = vmlaq_f32(sum, a2, vmovq_n_f32(b[2]));
        sum = vmlaq_f32(sum, a3, vmovq_n_f32(b[3]));
        vst1q_f32(&matrix->elements[8], sum);

        b = vld1q_f32(&rhs->elements[12]);
        sum = vmulq_f32(a0, vmovq_n_f32(b[0]));
        sum = vmlaq_f32(sum, a1, vmovq_n_f32(b[1]));
        sum = vmlaq_f32(sum, a2, vmovq_n_f32(b[2]));
        sum = vmlaq_f32(sum, a3, vmovq_n_f32(b[3]));
        vst1q_f32(&matrix->elements[12], sum);
    #elif defined ENABLE_SSE2_SIMD
        __m128 a0 = _mm_load_ps(&matrix->elements[0]);
        __m128 a1 = _mm_load_ps(&matrix->elements[4]);
        __m128 a2 = _mm_load_ps(&matrix->elements[8]);
        __m128 a3 = _mm_load_ps(&matrix->elements[12]);

        __m128 b = _mm_load_ps(&rhs->elements[0]);
        __m128 sum = _mm_mul_ps(a0, _mm_set1_ps(b[0]));
        sum = _mm_add_ps(sum, _mm_mul_ps(a1, _mm_set1_ps(b[1])));
        sum = _mm_add_ps(sum, _mm_mul_ps(a2, _mm_set1_ps(b[2])));
        sum = _mm_add_ps(sum, _mm_mul_ps(a3, _mm_set1_ps(b[3])));
        _mm_store_ps(&matrix->elements[0], sum);

        b = _mm_load_ps(&rhs->elements[4]);
        sum = _mm_mul_ps(a0, _mm_set1_ps(b[0]));
        sum = _mm_add_ps(sum, _mm_mul_ps(a1, _mm_set1_ps(b[1])));
        sum = _mm_add_ps(sum, _mm_mul_ps(a2, _mm_set1_ps(b[2])));
        sum = _mm_add_ps(sum, _mm_mul_ps(a3, _mm_set1_ps(b[3])));
        _mm_store_ps(&matrix->elements[4], sum);

        b = _mm_load_ps(&rhs->elements[8]);
        sum = _mm_mul_ps(a0, _mm_set1_ps(b[0]));
        sum = _mm_add_ps(sum, _mm_mul_ps(a1, _mm_set1_ps(b[1])));
        sum = _mm_add_ps(sum, _mm_mul_ps(a2, _mm_set1_ps(b[2])));
        sum = _mm_add_ps(sum, _mm_mul_ps(a3, _mm_set1_ps(b[3])));
        _mm_store_ps(&matrix->elements[8], sum);

        b = _mm_load_ps(&rhs->elements[12]);
        sum = _mm_mul_ps(a0, _mm_set1_ps(b[0]));
        sum = _mm_add_ps(_mm_mul_ps(a1, _mm_set1_ps(b[1])));
        sum = _mm_add_ps(_mm_mul_ps(a2, _mm_set1_ps(b[2])));
        sum = _mm_add_ps(_mm_mul_ps(a3, _mm_set1_ps(b[3])));
        _mm_store_ps(&matrix->elements[12], sum);
    #else
        float a00 = matrix->elements[0];
        float a01 = matrix->elements[1];
        float a02 = matrix->elements[2];
        float a03 = matrix->elements[3];
        float a10 = matrix->elements[4];
        float a11 = matrix->elements[5];
        float a12 = matrix->elements[6];
        float a13 = matrix->elements[7];
        float a20 = matrix->elements[8];
        float a21 = matrix->elements[9];
        float a22 = matrix->elements[10];
        float a23 = matrix->elements[11];
        float a30 = matrix->elements[12];
        float a31 = matrix->elements[13];
        float a32 = matrix->elements[14];
        float a33 = matrix->elements[15];

        float b0 = rhs->elements[0];
        float b1 = rhs->elements[1];
        float b2 = rhs->elements[2];
        float b3 = rhs->elements[3];
        matrix->elements[0] = b0 * a00 + b1 * a10 + b2 * a20 + b3 * a30;
        matrix->elements[1] = b0 * a01 + b1 * a11 + b2 * a21 + b3 * a31;
        matrix->elements[2] = b0 * a02 + b1 * a12 + b2 * a22 + b3 * a32;
        matrix->elements[3] = b0 * a03 + b1 * a13 + b2 * a23 + b3 * a33;

        b0 = rhs->elements[4];
        b1 = rhs->elements[5];
        b2 = rhs->elements[6];
        b3 = rhs->elements[7];
        matrix->elements[4] = b0 * a00 + b1 * a10 + b2 * a20 + b3 * a30;
        matrix->elements[5] = b0 * a01 + b1 * a11 + b2 * a21 + b3 * a31;
        matrix->elements[6] = b0 * a02 + b1 * a12 + b2 * a22 + b3 * a32;
        matrix->elements[7] = b0 * a03 + b1 * a13 + b2 * a23 + b3 * a33;

        b0 = rhs->elements[8];
        b1 = rhs->elements[9];
        b2 = rhs->elements[10];
        b3 = rhs->elements[11];
        matrix->elements[8] = b0 * a00 + b1 * a10 + b2 * a20 + b3 * a30;
        matrix->elements[9] = b0 * a01 + b1 * a11 + b2 * a21 + b3 * a31;
        matrix->elements[10] = b0 * a02 + b1 * a12 + b2 * a22 + b3 * a32;
        matrix->elements[11] = b0 * a03 + b1 * a13 + b2 * a23 + b3 * a33;

        b0 = rhs->elements[12];
        b1 = rhs->elements[13];
        b2 = rhs->elements[14];
        b3 = rhs->elements[15];
        matrix->elements[12] = b0 * a00 + b1 * a10 + b2 * a20 + b3 * a30;
        matrix->elements[13] = b0 * a01 + b1 * a11 + b2 * a21 + b3 * a31;
        matrix->elements[14] = b0 * a02 + b1 * a12 + b2 * a22 + b3 * a32;
        matrix->elements[15] = b0 * a03 + b1 * a13 + b2 * a23 + b3 * a33;
    #endif
}
