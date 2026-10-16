#include "cursor.h"

#include <gccore.h>
#include <wiiuse/wpad.h>

#include "canvas.h"
#include "cursors_texture.h"
#include "cursors_texture_tpl.h"

Cursor cursors[4] = {0};

static GXTexObj cursorTextures[4];

void cursor_init(void) {
    // Load cursor textures
    TPLFile cursorsTPL;
    TPL_OpenTPLFromMemory(&cursorsTPL, (void *)cursors_texture_tpl, cursors_texture_tpl_size);
    TPL_GetTexture(&cursorsTPL, cursors_player1, &cursorTextures[0]);
    TPL_GetTexture(&cursorsTPL, cursors_player2, &cursorTextures[1]);
    TPL_GetTexture(&cursorsTPL, cursors_player3, &cursorTextures[2]);
    TPL_GetTexture(&cursorsTPL, cursors_player4, &cursorTextures[3]);
}

void cursor_update(void) {
    WPAD_ScanPads();

    ir_t ir;
    WPAD_IR(0, &ir);
    cursors[0].x = ir.x;
    cursors[0].y = ir.y;

    uint32_t buttonsHeld = WPAD_ButtonsDown(0);
    cursors[0].click = (buttonsHeld & WPAD_BUTTON_A) != 0;
}

void cursor_render(void) {
    canvas_draw_image(&cursorTextures[0], cursors[0].x - 96 / 2, cursors[0].y - 96 / 2, 96, 96);
}
