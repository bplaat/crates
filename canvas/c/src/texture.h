#pragma once

#include <gccore.h>
#include <limits.h>
#include <malloc.h>
#include <ogc/cache.h>
#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

#define STBI_ONLY_PNG
#define STBI_NO_STDIO
#define STBI_NO_LINEAR
// Avoid incorrect vertical flips from stb_image's thread-local state on Wii.
#define STBI_NO_THREAD_LOCALS
#define STB_IMAGE_IMPLEMENTATION
#pragma GCC diagnostic push
#pragma GCC diagnostic ignored "-Wunused-function"
#include "stb_image.h"
#pragma GCC diagnostic pop

static bool texture_load_png_rgba8(GXTexObj* texture, const uint8_t* data, size_t size) {
    if (size > INT_MAX)
        return false;

    int width, height, channels;
    uint8_t* src = stbi_load_from_memory(data, (int)size, &width, &height, &channels, 4);
    if (!src)
        return false;
    if (width <= 0 || height <= 0 || width > 1024 || height > 1024) {
        stbi_image_free(src);
        return false;
    }

    int padded_width = (width + 3) & ~3;
    int padded_height = (height + 3) & ~3;
    size_t byte_count = (size_t)padded_width * padded_height * 4;
    uint8_t* dst = memalign(32, byte_count);
    if (!dst) {
        stbi_image_free(src);
        return false;
    }

    size_t pos = 0;
    for (int y = 0; y < padded_height; y += 4) {
        for (int x = 0; x < padded_width; x += 4) {
            for (int plane = 0; plane < 2; plane++) {
                for (int ry = 0; ry < 4; ry++) {
                    for (int rx = 0; rx < 4; rx++) {
                        if (x + rx < width && y + ry < height) {
                            size_t offset = ((size_t)(y + ry) * width + x + rx) * 4;
                            dst[pos++] = src[offset + (plane == 0 ? 3 : 1)];
                            dst[pos++] = src[offset + (plane == 0 ? 0 : 2)];
                        } else {
                            dst[pos++] = 0;
                            dst[pos++] = 0;
                        }
                    }
                }
            }
        }
    }

    stbi_image_free(src);
    DCFlushRange(dst, byte_count);
    GX_InitTexObj(texture, dst, width, height, GX_TF_RGBA8, GX_CLAMP, GX_CLAMP, GX_FALSE);
    return true;
}
