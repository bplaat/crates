/*
 * Copyright (c) 2021-2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

#include "prelude.h"

#ifndef _WIN32
char* strdup(const char* s) {
    char* n = malloc(strlen(s) + 1);
    strcpy(n, s);
    return n;
}
#endif

u32 fnv1a_32(const void* data, usize length) {
    u32 hash = 2166136261u;
    u8* ptr = (u8*)data;
    while (length--) {
        hash ^= *ptr++;
        hash *= 16777619u;
    }
    return hash;
}
