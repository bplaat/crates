#pragma once

#include <stdint.h>

typedef enum PageType {
    PAGE_MENU,
    PAGE_MAPS,
    PAGE_GAME,
    PAGE_HELP,
    PAGE_SETTINGS,
} PageType;

typedef struct Page {
    PageType type;
    int32_t focus;
    int32_t maxFocus;
} Page;

extern Page page;

void pages_goto(PageType pageType);

void pages_menu_render(void);

void pages_maps_render(void);

void pages_help_render(void);

void pages_settings_render(void);
