// PlaatCraft - Vector4 Math

#include "math/vector4.h"
#ifdef __ARM_NEON__
    #include <arm_neon.h>
#endif
#ifdef __SSE2__
    #include <emmintrin.h>
#endif

void vector4_add_vector4(Vector4* vector, Vector4* rhs) {
    #ifdef __ARM_NEON__
        vst1q_f32(&vector->x, vaddq_f32(vld1q_f32(&vector->x), vld1q_f32(&rhs->x)));
    #elif defined(__SSE2__)
        _mm_store_ps(&vector->x, _mm_add_ps(_mm_load_ps(&vector->x), _mm_load_ps(&rhs->x)));
    #else
        vector->x += rhs->x;
        vector->y += rhs->y;
        vector->z += rhs->z;
        vector->w += rhs->w;
    #endif
}

void vector4_mul_matrix4(Vector4* vector, Matrix4* rhs) {
    #ifdef __ARM_NEON__
        float32x4_t vec = vld1q_f32(&vector->x);
        float32x4_t sum = vmulq_f32(vld1q_f32(&rhs->elements[0]), vmovq_n_f32(vec[0]));
        sum = vmlaq_f32(sum, vld1q_f32(&rhs->elements[4]), vmovq_n_f32(vec[1]));
        sum = vmlaq_f32(sum, vld1q_f32(&rhs->elements[8]), vmovq_n_f32(vec[2]));
        sum = vmlaq_f32(sum, vld1q_f32(&rhs->elements[12]), vmovq_n_f32(vec[3]));
        vst1q_f32(&vector->x, sum);
    #elif defined(__SSE2__)
        __m128 vec = _mm_load_ps(&vector->x);
        __m128 sum = _mm_mul_ps(_mm_load_ps(&rhs->elements[0]), _mm_set1_ps(vec[0]));
        sum = _mm_add_ps(sum, _mm_mul_ps(_mm_load_ps(&rhs->elements[4]), _mm_set1_ps(vec[1])));
        sum = _mm_add_ps(sum, _mm_mul_ps(_mm_load_ps(&rhs->elements[8]), _mm_set1_ps(vec[2])));
        sum = _mm_add_ps(sum, _mm_mul_ps(_mm_load_ps(&rhs->elements[12]), _mm_set1_ps(vec[3])));
        _mm_store_ps(&vector->x, sum);
    #else
        float x = vector->x, y = vector->y, z = vector->z, w = vector->w;
        vector->x = rhs->elements[0] * x + rhs->elements[4] * y + rhs->elements[8] * z + rhs->elements[12] * w;
        vector->y = rhs->elements[1] * x + rhs->elements[5] * y + rhs->elements[9] * z + rhs->elements[13] * w;
        vector->z = rhs->elements[2] * x + rhs->elements[6] * y + rhs->elements[10] * z + rhs->elements[14] * w;
        vector->w = rhs->elements[3] * x + rhs->elements[7] * y + rhs->elements[11] * z + rhs->elements[15] * w;
    #endif
}
