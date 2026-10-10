// Small float math functions, accurate enough for games and written for the PowerPC 750:
// - Everything is single precision, converting integers to floats goes through double precision
//   and memory, these functions avoid it.
// - Divisions take about 17 cycles and multiplications 1 or 2, so constants are multiplied with
//   their reciprocal and the square root uses the reciprocal square root estimate instruction.

#include <math.h>
#include <stdbool.h>
#include <stdint.h>

#define PI 3.14159265358979f

// Converts an integer with |n| < 2^22 to a float: 1.5 * 2^23 + n is exact as float bits
static float small_int_to_float(int32_t n) {
    union {
        uint32_t i;
        float f;
    } bits = {0x4b400000u + (uint32_t)n};
    return bits.f - 12582912.0f;
}

float floorf(float x) {
    // Clear the fraction bits of the mantissa, rounding negative numbers down
    union {
        float f;
        uint32_t i;
    } bits = {x};
    int32_t exponent = (int32_t)((bits.i >> 23) & 255) - 127;
    if (exponent >= 23)  // Already an integer, infinite or NaN
        return x;
    if (exponent >= 0) {
        uint32_t fraction = 0x007fffffu >> exponent;
        if ((bits.i & fraction) == 0)
            return x;
        if (bits.i >> 31)
            bits.i += fraction;
        bits.i &= ~fraction;
    } else if (bits.i >> 31 == 0) {
        bits.i = 0;  // 0 <= x < 1
    } else if (bits.i << 1) {
        bits.f = -1.0f;  // -1 < x < 0
    }
    return bits.f;
}

float fmodf(float x, float y) {
    float quotient = x / y;
    return x - y * (quotient < 0.0f ? -floorf(-quotient) : floorf(quotient));
}

float fabsf(float x) {
    return __builtin_fabsf(x);
}

float sqrtf(float x) {
    if (x <= 0.0f || isinf(x))
        return x == 0.0f || isinf(x) ? x : __builtin_nanf("");
    // Estimate 1 / sqrt(x) to 5 bits, three Newton steps make that 24 bits, x / sqrt(x) = sqrt(x)
    float y;
    __asm__("frsqrte %0,%1" : "=f"(y) : "f"(x));
    float half = 0.5f * x;
    for (int i = 0; i < 3; i++)
        y = y * (1.5f - half * y * y);
    return x * y;
}

float hypotf(float x, float y) {
    return sqrtf(x * x + y * y);
}

float sinf(float x) {
    // Reduce to [-pi, pi], then fold into [-pi/2, pi/2]
    x -= 2.0f * PI * floorf(x * (0.5f / PI) + 0.5f);
    if (x > PI / 2.0f)
        x = PI - x;
    else if (x < -PI / 2.0f)
        x = -PI - x;
    // Taylor series up to x^11
    float x2 = x * x;
    return x * (1.0f + x2 * (-1.0f / 6.0f +
                             x2 * (1.0f / 120.0f +
                                   x2 * (-1.0f / 5040.0f + x2 * (1.0f / 362880.0f + x2 * (-1.0f / 39916800.0f))))));
}

float cosf(float x) {
    return sinf(x + PI / 2.0f);
}

float tanf(float x) {
    return sinf(x) / cosf(x);
}

float atanf(float x) {
    // Abramowitz and Stegun 4.4.49 for |x| <= 1
    bool invert = fabsf(x) > 1.0f;
    if (invert)
        x = 1.0f / x;
    float x2 = x * x;
    float y =
        x *
        (0.9999993329f +
         x2 * (-0.3332985605f +
               x2 * (0.1994653599f + x2 * (-0.1390853351f +
                                           x2 * (0.0964200441f + x2 * (-0.0559098861f +
                                                                       x2 * (0.0218612288f + x2 * -0.0040540580f)))))));
    if (invert)
        y = (x > 0.0f ? PI / 2.0f : -PI / 2.0f) - y;
    return y;
}

float atan2f(float y, float x) {
    if (x > 0.0f)
        return atanf(y / x);
    if (x < 0.0f)
        return atanf(y / x) + (y >= 0.0f ? PI : -PI);
    if (y > 0.0f)
        return PI / 2.0f;
    if (y < 0.0f)
        return -PI / 2.0f;
    return 0.0f;
}

// 2^x as 2^i * e^(f * ln 2), the fraction is done with a Taylor series of e^x
float exp2f(float x) {
    if (isnan(x))
        return x;
    if (x >= 128.0f)
        return __builtin_inff();
    if (x < -126.0f)
        return 0.0f;
    float i = floorf(x + 0.5f);
    float f = (x - i) * 0.69314718f;
    float y =
        1.0f + f * (1.0f + f * (1.0f / 2.0f +
                                f * (1.0f / 6.0f + f * (1.0f / 24.0f + f * (1.0f / 120.0f + f * (1.0f / 720.0f))))));
    // x rounds up to 128 above 127.5, 2^128 doesn't fit in a float so y gets that last factor 2
    if (i > 127.0f) {
        i = 127.0f;
        y *= 2.0f;
    }
    union {
        uint32_t i;
        float f;
    } scale = {(uint32_t)((int32_t)i + 127) << 23};
    return y * scale.f;
}

// log2(x), split x into 2^e * m and use the atanh series for log(m)
float log2f(float x) {
    if (!(x > 0.0f))
        return x == 0.0f ? -__builtin_inff() : __builtin_nanf("");
    if (isinf(x))
        return x;
    union {
        float f;
        uint32_t i;
    } bits = {x};
    int32_t e = (int32_t)((bits.i >> 23) & 255) - 127;
    bits.i = (bits.i & 0x007fffff) | 0x3f800000;
    float m = bits.f;
    if (m > 1.41421356f) {
        m *= 0.5f;
        e++;
    }
    float s = (m - 1.0f) / (m + 1.0f);
    float s2 = s * s;
    float ln = 2.0f * s * (1.0f + s2 * (1.0f / 3.0f + s2 * (1.0f / 5.0f + s2 * (1.0f / 7.0f + s2 * (1.0f / 9.0f)))));
    return small_int_to_float(e) + ln * 1.44269504f;
}

float powf(float x, float y) {
    if (y == 0.0f || x == 1.0f)
        return 1.0f;
    if (x == 0.0f)
        return y > 0.0f ? 0.0f : __builtin_inff();
    if (x < 0.0f) {
        // Only integer powers of negative numbers are real
        if (floorf(y) != y)
            return __builtin_nanf("");
        float result = powf(-x, y);
        return fabsf(fmodf(y, 2.0f)) == 1.0f ? -result : result;
    }
    return exp2f(y * log2f(x));
}
