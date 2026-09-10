#include "pages.h"

#include <stdlib.h>
#include <gccore.h>

#include "canvas.h"
#include "cursor.h"

Page page = {.type = PAGE_MENU, .focus = -1, .maxFocus = 4};

void pages_goto(PageType pageType) {
    page.type = pageType;
    page.focus = -1;
}

void pages_menu_render(void) {
    int32_t x = 150;
    int32_t y = 50;
    canvas_fill_text(x, y, "PlaatCraft", 32, 0xffffffff);
    y += 32 + 16;

    if (cursors[0].x >= x && cursors[0].x < x + 200 && cursors[0].y >= y && cursors[0].y < y + 52) page.focus = 0;
    canvas_fill_rect(x, y, 200, 52, page.focus == 0 ? 0xccccccff : 0xffffffff);
    canvas_fill_text(x + 16, y + (52 - 16) / 2, "Play", 16, 0x000000ff);
    if (page.focus == 0 && cursors[0].click) pages_goto(PAGE_MAPS);
    y += 52 + 16;

    if (cursors[0].x >= x && cursors[0].x < x + 200 && cursors[0].y >= y && cursors[0].y < y + 52) page.focus = 1;
    canvas_fill_rect(x, y, 200, 52, page.focus == 1 ? 0xccccccff : 0xffffffff);
    canvas_fill_text(x + 16, y + (52 - 16) / 2, "Help", 16, 0x000000ff);
    if (page.focus == 1 && cursors[0].click) pages_goto(PAGE_HELP);
    y += 52 + 16;

    if (cursors[0].x >= x && cursors[0].x < x + 200 && cursors[0].y >= y && cursors[0].y < y + 52) page.focus = 2;
    canvas_fill_rect(x, y, 200, 52, page.focus == 2 ? 0xccccccff : 0xffffffff);
    canvas_fill_text(x + 16, y + (52 - 16) / 2, "Settings", 16, 0x000000ff);
    if (page.focus == 2 && cursors[0].click) pages_goto(PAGE_SETTINGS);
    y += 52 + 16;

    if (cursors[0].x >= x && cursors[0].x < x + 200 && cursors[0].y >= y && cursors[0].y < y + 52) page.focus = 3;
    canvas_fill_rect(x, y, 200, 52, page.focus == 3 ? 0xccccccff : 0xffffffff);
    canvas_fill_text(x + 16, y + (52 - 16) / 2, "Exit", 16, 0x000000ff);
    if (page.focus == 3 && cursors[0].click) {
        SYS_ResetSystem(SYS_POWEROFF, 0, 0);
        exit(0);
    }
    y += 52 + 16;

    canvas_fill_text(x, y, "Made by PlaatSoft", 8, 0xffffffff);
}

void pages_maps_render(void) {
    int32_t x = 150;
    int32_t y = 32;
    canvas_fill_text(x, y, "Maps", 32, 0xffffffff);
    y += 32 + 16;

    if (cursors[0].x >= x && cursors[0].x < x + 200 && cursors[0].y >= y && cursors[0].y < y + 52) page.focus = 0;
    canvas_fill_rect(x, y, 200, 52, page.focus == 0 ? 0xccccccff : 0xffffffff);
    canvas_fill_text(x + 16, y + (52 - 16) / 2, "Create world", 16, 0x000000ff);
    if (page.focus == 0 && cursors[0].click) pages_goto(PAGE_GAME);
    y += 52 + 16;

    if (cursors[0].x >= x && cursors[0].x < x + 200 && cursors[0].y >= y && cursors[0].y < y + 52) page.focus = 1;
    canvas_fill_rect(x, y, 200, 52, page.focus == 1 ? 0xccccccff : 0xffffffff);
    canvas_fill_text(x + 16, y + (52 - 16) / 2, "Back", 16, 0x000000ff);
    if (page.focus == 1 && cursors[0].click) pages_goto(PAGE_MENU);
    y += 52 + 16;
}

void pages_help_render(void) {
    int32_t x = 32;
    int32_t y = 32;
    canvas_fill_text(x, y, "Help", 32, 0xffffffff);
    y += 32 + 16;

    canvas_fill_text(x, y, "PlaatCraft is a Minecraft like", 16, 0xffffffff);
    y += 16 + 8;
    canvas_fill_text(x, y, "sandbox game. You can create a", 16, 0xffffffff);
    y += 16 + 8;
    canvas_fill_text(x, y, "new world a place blocks. The", 16, 0xffffffff);
    y += 16 + 8;
    canvas_fill_text(x, y, "game is in development so expect", 16, 0xffffffff);
    y += 16 + 8;
    canvas_fill_text(x, y, "not much.", 16, 0xffffffff);
    y += 16 + 16;

    if (cursors[0].x >= x && cursors[0].x < x + 200 && cursors[0].y >= y && cursors[0].y < y + 52) page.focus = 0;
    canvas_fill_rect(x, y, 200, 52, page.focus == 0 ? 0xccccccff : 0xffffffff);
    canvas_fill_text(x + 16, y + (52 - 16) / 2, "Back", 16, 0x000000ff);
    if (page.focus == 0 && cursors[0].click) pages_goto(PAGE_MENU);
    y += 52 + 16;
}

void pages_settings_render(void) {
    int32_t x = 150;
    int32_t y = 32;
    canvas_fill_text(x, y, "Settings", 32, 0xffffffff);
    y += 32 + 16;

    if (cursors[0].x >= x && cursors[0].x < x + 200 && cursors[0].y >= y && cursors[0].y < y + 52) page.focus = 0;
    canvas_fill_rect(x, y, 200, 52, page.focus == 0 ? 0xccccccff : 0xffffffff);
    canvas_fill_text(x + 16, y + (52 - 16) / 2, "Back", 16, 0x000000ff);
    if (page.focus == 0 && cursors[0].click) pages_goto(PAGE_MENU);
    y += 52 + 16;
}
