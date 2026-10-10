#pragma once

#define isnan(x) __builtin_isnan(x)
#define isinf(x) __builtin_isinf(x)
#define isfinite(x) __builtin_isfinite(x)

float floorf(float x);
float fabsf(float x);
float fmodf(float x, float y);
float sqrtf(float x);
float hypotf(float x, float y);
float sinf(float x);
float cosf(float x);
float tanf(float x);
float atanf(float x);
float atan2f(float y, float x);
float exp2f(float x);
float log2f(float x);
float powf(float x, float y);

// Double precision math is not supported, use the float versions
#define sin(x) sinf(x)
#define cos(x) cosf(x)
#define atan2(y, x) atan2f(y, x)
#define pow(x, y) powf(x, y)
