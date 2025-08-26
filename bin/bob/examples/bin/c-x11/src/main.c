/*
 * Copyright (c) 2022-2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

// A simple pure C X11 client example that creates a window and draws some things

#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "x11.h"

int main(void) {
    x11_connection_t conn;
    if (!x11_connect(&conn)) {
        fprintf(stderr, "Can't connect to X11 display\n");
        return EXIT_FAILURE;
    }
    printf("Screen: %dx%d\n", conn.screen.width_in_pixels, conn.screen.height_in_pixels);

    // Create font
    uint32_t font = x11_generate_id(&conn);
    char* font_name = "fixed";
    x11_open_font(&conn, font, font_name, strlen(font_name));

    // Create gc
    uint32_t gc = x11_generate_id(&conn);
    uint32_t create_gc_list[] = {conn.screen.black_pixel, conn.screen.white_pixel, font};
    x11_create_gc(&conn, gc, conn.screen.root, X11_GC_FOREGROUND | X11_GC_BACKGROUND | X11_GC_FONT, create_gc_list,
                  sizeof(create_gc_list));
    x11_close_font(&conn, font);

    // Create window
    char* window_title = "Hello X11!";
    int32_t window_width = 640;
    int32_t window_height = 480;

    uint32_t window = x11_generate_id(&conn);
    uint32_t create_window_list[] = {conn.screen.white_pixel, X11_EVENT_MASK_EXPOSURE};
    x11_create_window(&conn, X11_COPY_FROM_PARENT, window, conn.screen.root, 0, 0, window_width, window_height, 0,
                      X11_WINDOW_CLASS_INPUT_OUTPUT, conn.screen.root_visual, X11_CW_BACK_PIXEL | X11_CW_EVENT_MASK,
                      create_window_list, sizeof(create_window_list));

    x11_change_property(&conn, X11_PROP_MODE_REPLACE, window, X11_ATOM_WM_NAME, X11_ATOM_STRING, 8, window_title,
                        strlen(window_title));

    uint32_t configure_list[] = {window_width, window_height};
    x11_configure_window(&conn, window, X11_CONFIG_WINDOW_WIDTH | X11_CONFIG_WINDOW_HEIGHT, configure_list,
                         sizeof(configure_list));

    x11_map_window(&conn, window);

    // Event loop
    x11_event_t event;
    while (x11_wait_for_event(&conn, &event)) {
        // Draw event
        if (event.type == X11_EXPOSE) {
            x11_rectangle_t rectangles[] = {{55, 55, 50, 50}, {75, 75, 50, 50}, {95, 95, 50, 50}};
            x11_poly_rectangle(&conn, window, gc, rectangles, sizeof(rectangles));

            char* message = "Hello World from an X11 window!";
            x11_image_text_8(&conn, window, gc, 16, 16, message, strlen(message));
        }
    }

    x11_disconnect(&conn);
    return EXIT_SUCCESS;
}
