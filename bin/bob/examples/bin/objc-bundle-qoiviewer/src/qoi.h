/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 *
 * Minimal QOI (Quite OK Image) decoder -- single-header, pure C, no dependencies.
 * Spec: https://qoiformat.org/qoi-specification.pdf
 *
 * Usage:
 *   #define QOI_IMPLEMENTATION
 *   #include "qoi.h"
 *
 * Returns a heap-allocated RGBA pixel buffer that the caller must free(),
 * or NULL on error. Width/height/channels are written to the out-parameters.
 */

#ifndef QOI_H
#define QOI_H

#include <stdint.h>

typedef struct {
    uint32_t width;
    uint32_t height;
    uint8_t channels;   /* 3 = RGB, 4 = RGBA */
    uint8_t colorspace; /* 0 = sRGB with linear alpha, 1 = all channels linear */
} QoiDesc;

/* Decode QOI data. Returns RGBA pixel buffer (caller owns, free with free()) or NULL on error. */
uint8_t* qoi_decode(const void* data, int size, QoiDesc* desc);

#ifdef QOI_IMPLEMENTATION

#include <stdlib.h>
#include <string.h>

typedef struct { uint8_t r, g, b, a; } QoiRgba;

static int qoi_hash(QoiRgba c) {
    return (c.r * 3 + c.g * 5 + c.b * 7 + c.a * 11) & 63;
}

uint8_t* qoi_decode(const void* data, int size, QoiDesc* desc) {
    if (!data || !desc || size < 22) return NULL; /* 14 header + 8 end marker */

    const uint8_t* b = (const uint8_t*)data;

    /* Validate magic "qoif" */
    if (b[0] != 'q' || b[1] != 'o' || b[2] != 'i' || b[3] != 'f') return NULL;

    /* Read header (big-endian) */
    desc->width      = ((uint32_t)b[4]  << 24) | ((uint32_t)b[5]  << 16) |
                       ((uint32_t)b[6]  <<  8) |  (uint32_t)b[7];
    desc->height     = ((uint32_t)b[8]  << 24) | ((uint32_t)b[9]  << 16) |
                       ((uint32_t)b[10] <<  8) |  (uint32_t)b[11];
    desc->channels   = b[12];
    desc->colorspace = b[13];

    if (desc->width == 0 || desc->height == 0) return NULL;
    if (desc->channels < 3 || desc->channels > 4) return NULL;

    /* Guard against width*height overflow before allocating */
    uint64_t px_count64 = (uint64_t)desc->width * desc->height;
    if (px_count64 > 400000000u) return NULL; /* sanity: >400 MP is unreasonable */
    int px_count = (int)px_count64;

    /* Allocate zero-initialised output (safe if we break early) */
    uint8_t* out = (uint8_t*)calloc((size_t)px_count, 4);
    if (!out) return NULL;

    QoiRgba index[64];
    memset(index, 0, sizeof(index));

    QoiRgba px = { 0, 0, 0, 255 };
    int run = 0;
    int p   = 14;           /* read cursor, starts after header */
    int end = size - 8;     /* end marker begins here; valid data: [14, end) */

    for (int i = 0; i < px_count; i++) {
        if (run > 0) {
            /* Still inside a run -- just repeat the current pixel */
            run--;
        } else if (p < end) {
            uint8_t tag = b[p++];

            if (tag == 0xFE) {             /* QOI_OP_RGB: needs 3 more bytes */
                if (end - p < 3) { free(out); return NULL; }
                px.r = b[p++];
                px.g = b[p++];
                px.b = b[p++];
                /* alpha unchanged */

            } else if (tag == 0xFF) {      /* QOI_OP_RGBA: needs 4 more bytes */
                if (end - p < 4) { free(out); return NULL; }
                px.r = b[p++];
                px.g = b[p++];
                px.b = b[p++];
                px.a = b[p++];

            } else if ((tag & 0xC0) == 0x00) { /* QOI_OP_INDEX */
                px = index[tag & 0x3F];

            } else if ((tag & 0xC0) == 0x40) { /* QOI_OP_DIFF */
                /* Each channel diff: 2-bit value with bias 2, range -2..1 */
                px.r = (uint8_t)(px.r + ((tag >> 4) & 0x03) - 2);
                px.g = (uint8_t)(px.g + ((tag >> 2) & 0x03) - 2);
                px.b = (uint8_t)(px.b + ( tag        & 0x03) - 2);
                /* alpha unchanged */

            } else if ((tag & 0xC0) == 0x80) { /* QOI_OP_LUMA: needs 1 more byte */
                if (p >= end) { free(out); return NULL; }
                uint8_t b2 = b[p++];
                /* dg: 6-bit with bias 32, range -32..31 */
                int dg = (tag & 0x3F) - 32;
                /* dr and db are relative to dg; 4-bit with bias 8, range -8..7 */
                px.r = (uint8_t)(px.r + dg + ((b2 >> 4) & 0x0F) - 8);
                px.g = (uint8_t)(px.g + dg);
                px.b = (uint8_t)(px.b + dg + ( b2        & 0x0F) - 8);
                /* alpha unchanged */

            } else {                           /* QOI_OP_RUN (tag & 0xC0 == 0xC0) */
                /* 6-bit run length with bias -1, range 1..62 (stored as 0..61) */
                /* Emit 1 pixel now; the remaining (stored_val) are handled above */
                run = tag & 0x3F;
            }

            /* Every decoded pixel (including first of a run) updates the index */
            index[qoi_hash(px)] = px;
        }

        /* Write RGBA pixel */
        int off = i * 4;
        out[off]     = px.r;
        out[off + 1] = px.g;
        out[off + 2] = px.b;
        out[off + 3] = px.a;
    }

    return out;
}

#endif /* QOI_IMPLEMENTATION */
#endif /* QOI_H */
