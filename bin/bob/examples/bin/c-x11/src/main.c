/*
 * Copyright (c) 2022-2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

// A simple pure C X11 client example that uses a canvas software renderer and blits to the window

#include <signal.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

#include "canvas.h"
#include "x11.h"

static void render(canvas_t* canvas) {
    // Clear to white
    canvas_fill_rect(canvas, 0, 0, canvas->width, canvas->height, CANVAS_COLOR(255, 255, 255));

    // Draw filled rectangles
    canvas_fill_rect(canvas, 40, 40, 120, 80, CANVAS_COLOR(255, 0, 0));
    canvas_fill_rect(canvas, 200, 40, 120, 80, CANVAS_COLOR(0, 255, 0));
    canvas_fill_rect(canvas, 360, 40, 120, 80, CANVAS_COLOR(0, 0, 255));

    // Draw outlined rectangles
    canvas_stroke_rect(canvas, 40, 160, 120, 80, CANVAS_COLOR(255, 0, 0));
    canvas_stroke_rect(canvas, 200, 160, 120, 80, CANVAS_COLOR(0, 255, 0));
    canvas_stroke_rect(canvas, 360, 160, 120, 80, CANVAS_COLOR(0, 0, 255));

    // Nested stroke rects as a simple pattern
    for (int32_t i = 0; i < 5; i++) {
        canvas_stroke_rect(canvas, 40 + i * 12, 280 + i * 8, 200 - i * 24, 120 - i * 16,
                           CANVAS_COLOR(80 + i * 30, 80 + i * 20, 200 - i * 30));
    }
}

int main(void) {
    signal(SIGPIPE, SIG_IGN);

    x11_connection_t conn;
    if (!x11_connect(&conn)) {
        fprintf(stderr, "Can't connect to X11 display\n");
        return EXIT_FAILURE;
    }
    printf("Screen: %dx%d, MIT-SHM: %s\n", conn.screen.width_in_pixels, conn.screen.height_in_pixels,
           conn.has_shm ? "yes" : "no");

    int32_t window_width = 640;
    int32_t window_height = 480;
    int32_t window_x = (conn.screen.width_in_pixels - window_width) / 2;
    int32_t window_y = (conn.screen.height_in_pixels - window_height) / 2;

    // Create window — BackPixel = white so any server-side clear during resize
    // is the same colour as the canvas background, making the flash invisible.
    const char* window_title = "Hello Canvas!";
    uint32_t window = x11_generate_id(&conn);
    uint32_t create_window_list[] = {conn.screen.white_pixel,
                                     X11_EVENT_MASK_EXPOSURE | X11_EVENT_MASK_STRUCTURE_NOTIFY};
    x11_create_window(&conn, X11_COPY_FROM_PARENT, window, conn.screen.root, window_x, window_y, window_width,
                      window_height, 0, X11_WINDOW_CLASS_INPUT_OUTPUT, conn.screen.root_visual,
                      X11_CW_BACK_PIXEL | X11_CW_EVENT_MASK, create_window_list, sizeof(create_window_list));

    // Set ICCCM properties
    x11_change_property(&conn, X11_PROP_MODE_REPLACE, window, X11_ATOM_WM_NAME, X11_ATOM_STRING, 8, (void*)window_title,
                        strlen(window_title));
    x11_change_property(&conn, X11_PROP_MODE_REPLACE, window, conn.net_wm_name, conn.utf8_string, 8,
                        (void*)window_title, strlen(window_title));

    const char wm_class[] = "canvas-example\0Canvas-Example";
    x11_change_property(&conn, X11_PROP_MODE_REPLACE, window, X11_ATOM_WM_CLASS, X11_ATOM_STRING, 8, (void*)wm_class,
                        sizeof(wm_class));

    char hostname[256];
    if (gethostname(hostname, sizeof(hostname)) == 0) {
        hostname[sizeof(hostname) - 1] = '\0';
        x11_change_property(&conn, X11_PROP_MODE_REPLACE, window, X11_ATOM_WM_CLIENT_MACHINE, X11_ATOM_STRING, 8,
                            hostname, strlen(hostname));
    }

    uint32_t pid = (uint32_t)getpid();
    x11_change_property(&conn, X11_PROP_MODE_REPLACE, window, conn.net_wm_pid, X11_ATOM_CARDINAL, 32, &pid,
                        sizeof(pid));

    x11_set_wm_protocols(&conn, window);
    x11_set_wm_hints(&conn, window);

    uint32_t size_hints[18] = {0};
    size_hints[0] = 4 | 8;  // PPosition | PSize
    size_hints[1] = (uint32_t)window_x;
    size_hints[2] = (uint32_t)window_y;
    size_hints[3] = (uint32_t)window_width;
    size_hints[4] = (uint32_t)window_height;
    x11_change_property(&conn, X11_PROP_MODE_REPLACE, window, X11_ATOM_WM_NORMAL_HINTS, X11_ATOM_WM_SIZE_HINTS, 32,
                        size_hints, sizeof(size_hints));

    x11_map_window(&conn, window);

    // Set _NET_WM_WINDOW_TYPE for modern compositors (GNOME, KDE, etc.)
    x11_change_property(&conn, X11_PROP_MODE_REPLACE, window, conn.net_wm_window_type, X11_ATOM_ATOM, 32,
                        &conn.net_wm_window_type_normal, sizeof(conn.net_wm_window_type_normal));

    // Create image and canvas backed by it
    x11_image_t img;
    if (!x11_create_image(&conn, &img, window, window_width, window_height)) {
        fprintf(stderr, "Can't create image\n");
        x11_disconnect(&conn);
        return EXIT_FAILURE;
    }

    canvas_t canvas;
    canvas_init(&canvas, img.width, img.height, img.pixels);

    // Event loop
    x11_event_t event;
    bool running = true;
    while (running && x11_wait_for_event(&conn, &event)) {
        if (event.type == X11_CONFIGURE_NOTIFY) {
            int32_t new_w = (int32_t)event.configure_width;
            int32_t new_h = (int32_t)event.configure_height;
            if (new_w > 0 && new_h > 0 && (new_w != img.width || new_h != img.height)) {
                if (!x11_resize_image(&conn, &img, new_w, new_h)) {
                    fprintf(stderr, "Can't resize image\n");
                    running = false;
                    break;
                }
                canvas_init(&canvas, img.width, img.height, img.pixels);
                render(&canvas);
                x11_put_image(&conn, window, &img);
            }
        }

        // Expose (count == 0 means no more expose events pending): render and blit.
        if (event.type == X11_EXPOSE && event.expose_count == 0) {
            render(&canvas);
            x11_put_image(&conn, window, &img);
        }

        if (event.type == X11_CLIENT_MESSAGE_CLOSE) {
            running = false;
        }
    }

    x11_destroy_image(&conn, &img);
    x11_disconnect(&conn);
    return EXIT_SUCCESS;
}
