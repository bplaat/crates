#pragma once

#include <stdbool.h>
#include <stdint.h>
#include <wiiuse/wpad.h>

#include "canvas.h"

typedef struct Cursor {
    bool enabled;
    float x;
    float y;
    float angle;
    uint32_t buttons_down;
    uint32_t buttons_held;
    uint32_t buttons_up;
    GXTexObj texture;
} Cursor;

static const uint8_t cursor1_png[] = {
#embed "assets/cursor1.png"
};
static const uint8_t cursor2_png[] = {
#embed "assets/cursor2.png"
};
static const uint8_t cursor3_png[] = {
#embed "assets/cursor3.png"
};
static const uint8_t cursor4_png[] = {
#embed "assets/cursor4.png"
};

static Cursor cursors[4] = {0};

static bool cursor_init(void) {
    if (!texture_load_png_rgba8(&cursors[0].texture, cursor1_png, sizeof(cursor1_png)) ||
        !texture_load_png_rgba8(&cursors[1].texture, cursor2_png, sizeof(cursor2_png)) ||
        !texture_load_png_rgba8(&cursors[2].texture, cursor3_png, sizeof(cursor3_png)) ||
        !texture_load_png_rgba8(&cursors[3].texture, cursor4_png, sizeof(cursor4_png)))
        return false;

    // Read cursors state
    WPAD_ScanPads();
    for (int32_t i = 0; i < 4; i++) {
        Cursor* cursor = &cursors[i];
        // uint32_t devtype;
        // WPAD_Probe(i, &devtype);
        // cursor->enabled = devtype == WPAD_EXP_NONE || devtype == WPAD_EXP_NUNCHUK || devtype == WPAD_EXP_CLASSIC;
        cursor->enabled = i == 0;
    }
    return true;
}

static void cursor_update(void) {
    // Read cursors state
    WPAD_ScanPads();
    for (int32_t i = 0; i < 4; i++) {
        Cursor* cursor = &cursors[i];
        if (cursor->enabled) {
            ir_t ir;
            WPAD_IR(i, &ir);
            cursor->x = ir.x;
            cursor->y = ir.y;
            cursor->angle = ir.angle;
            cursor->buttons_down = WPAD_ButtonsDown(i);
            cursor->buttons_held = WPAD_ButtonsHeld(i);
            cursor->buttons_up = WPAD_ButtonsUp(i);
        }
    }
}

static void cursor_render(void) {
    // Draw enabled cursors on screen
    for (int32_t i = 0; i < 4; i++) {
        Cursor* cursor = &cursors[i];
        if (cursor->enabled) {
            ps_guMtxRotRad(canvas.transform_matrix, 'z', DegToRad(cursor->angle));
            canvas_draw_image(&cursor->texture, cursor->x - 96 / 2, cursor->y - 96 / 2, 96, 96, 0xffffffff);
        }
    }
    ps_guMtxIdentity(canvas.transform_matrix);
}
