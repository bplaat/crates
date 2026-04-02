/*
 * Copyright (c) 2022-2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

// A simple pure C X11 client example that creates a window and draws some things using X11

#include <stdbool.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

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
    const char* font_name = "fixed";
    x11_open_font(&conn, font, font_name, strlen(font_name));

    // Create gc
    uint32_t gc = x11_generate_id(&conn);
    uint32_t create_gc_list[] = {conn.screen.black_pixel, conn.screen.white_pixel, font};
    x11_create_gc(&conn, gc, conn.screen.root, X11_GC_FOREGROUND | X11_GC_BACKGROUND | X11_GC_FONT, create_gc_list,
                  sizeof(create_gc_list));
    x11_close_font(&conn, font);

    // Create window
    const char* window_title = "Hello X11!";
    int32_t window_width = 640;
    int32_t window_height = 480;
    int32_t window_x = (conn.screen.width_in_pixels - window_width) / 2;
    int32_t window_y = (conn.screen.height_in_pixels - window_height) / 2;

    uint32_t window = x11_generate_id(&conn);
    uint32_t create_window_list[] = {conn.screen.white_pixel, X11_EVENT_MASK_EXPOSURE};
    x11_create_window(&conn, X11_COPY_FROM_PARENT, window, conn.screen.root, window_x, window_y, window_width,
                      window_height, 0, X11_WINDOW_CLASS_INPUT_OUTPUT, conn.screen.root_visual,
                      X11_CW_BACK_PIXEL | X11_CW_EVENT_MASK, create_window_list, sizeof(create_window_list));

    // Set ICCCM properties
    x11_change_property(&conn, X11_PROP_MODE_REPLACE, window, X11_ATOM_WM_NAME, X11_ATOM_STRING, 8, (void*)window_title,
                        strlen(window_title));
    x11_change_property(&conn, X11_PROP_MODE_REPLACE, window, conn.net_wm_name, conn.utf8_string, 8,
                        (void*)window_title, strlen(window_title));

    // WM_CLASS: "x11-example\0X11-Example\0" (instance\0class\0)
    const char wm_class[] = "x11-example\0X11-Example";
    x11_change_property(&conn, X11_PROP_MODE_REPLACE, window, X11_ATOM_WM_CLASS, X11_ATOM_STRING, 8, (void*)wm_class,
                        sizeof(wm_class));

    // WM_CLIENT_MACHINE
    char hostname[256];
    if (gethostname(hostname, sizeof(hostname)) == 0) {
        hostname[sizeof(hostname) - 1] = '\0';
        x11_change_property(&conn, X11_PROP_MODE_REPLACE, window, X11_ATOM_WM_CLIENT_MACHINE, X11_ATOM_STRING, 8,
                            hostname, strlen(hostname));
    }

    // _NET_WM_PID
    uint32_t pid = getpid();
    x11_change_property(&conn, X11_PROP_MODE_REPLACE, window, conn.net_wm_pid, X11_ATOM_CARDINAL, 32, &pid,
                        sizeof(pid));

    x11_set_wm_protocols(&conn, window);
    x11_set_wm_hints(&conn, window);

    // Set WM_NORMAL_HINTS with PPosition | PSize so the window manager honors our position
    uint32_t size_hints[18] = {0};
    size_hints[0] = 4 | 8;                    // flags: PPosition | PSize
    size_hints[1] = (uint32_t)window_x;       // x
    size_hints[2] = (uint32_t)window_y;       // y
    size_hints[3] = (uint32_t)window_width;   // width
    size_hints[4] = (uint32_t)window_height;  // height
    x11_change_property(&conn, X11_PROP_MODE_REPLACE, window, X11_ATOM_WM_NORMAL_HINTS, X11_ATOM_WM_SIZE_HINTS, 32,
                        size_hints, sizeof(size_hints));

    x11_map_window(&conn, window);

    // Event loop
    x11_event_t event;
    bool running = true;
    while (running && x11_wait_for_event(&conn, &event)) {
        if (event.type == X11_EXPOSE && event.expose_count == 0) {
            x11_rectangle_t rectangles[] = {{55, 55, 50, 50}, {75, 75, 50, 50}, {95, 95, 50, 50}};
            x11_poly_rectangle(&conn, window, gc, rectangles, sizeof(rectangles));

            const char* message = "Hello World from an X11 window!";
            x11_image_text_8(&conn, window, gc, 16, 16, message, strlen(message));
        }
        if (event.type == X11_CLIENT_MESSAGE_CLOSE) {
            running = false;
        }
    }

    x11_disconnect(&conn);
    return EXIT_SUCCESS;
}
