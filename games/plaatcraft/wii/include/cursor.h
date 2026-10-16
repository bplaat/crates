#pragma once

#include <stdbool.h>
#include <stdint.h>

typedef struct Cursor {
    int32_t x;
    int32_t y;
    bool click;
} Cursor;

extern Cursor cursors[4];

void cursor_init(void);

void cursor_update(void);

void cursor_render(void);
