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
    canvas_fill_rect(canvas, 0.0f, 0.0f, (float)canvas->width, (float)canvas->height, CANVAS_COLOR(255, 255, 255));

    // Draw filled rectangles
    canvas_fill_rect(canvas, 40.0f, 40.0f, 120.0f, 80.0f, CANVAS_COLOR(255, 0, 0));
    canvas_fill_rect(canvas, 200.0f, 40.0f, 120.0f, 80.0f, CANVAS_COLOR(0, 255, 0));
    canvas_fill_rect(canvas, 360.0f, 40.0f, 120.0f, 80.0f, CANVAS_COLOR(0, 0, 255));

    // Draw outlined rectangles
    canvas_stroke_rect(canvas, 40.0f, 160.0f, 120.0f, 80.0f, 4.0f, CANVAS_COLOR(255, 0, 0));
    canvas_stroke_rect(canvas, 200.0f, 160.0f, 120.0f, 80.0f, 4.0f, CANVAS_COLOR(0, 255, 0));
    canvas_stroke_rect(canvas, 360.0f, 160.0f, 120.0f, 80.0f, 4.0f, CANVAS_COLOR(0, 0, 255));

    // Nested stroke rects as a simple pattern
    for (int32_t i = 0; i < 5; i++) {
        canvas_stroke_rect(canvas, 40.0f + i * 12.0f, 280.0f + i * 8.0f, 200.0f - i * 24.0f, 120.0f - i * 16.0f, 1.0f,
                           CANVAS_COLOR(80 + i * 30, 80 + i * 20, 200 - i * 30));
    }
}

// Find the monitor with the greatest overlap with the window rect.
// Falls back to the first monitor if none overlap.
static int32_t find_monitor_for_window(x11_monitor_t* monitors, int32_t monitor_count, int32_t wx, int32_t wy,
                                       int32_t ww, int32_t wh) {
    int32_t best = 0;
    int32_t best_area = -1;
    for (int32_t i = 0; i < monitor_count; i++) {
        int32_t ox1 = wx > monitors[i].x ? wx : monitors[i].x;
        int32_t oy1 = wy > monitors[i].y ? wy : monitors[i].y;
        int32_t ox2 = (wx + ww) < (monitors[i].x + monitors[i].width) ? (wx + ww) : (monitors[i].x + monitors[i].width);
        int32_t oy2 =
            (wy + wh) < (monitors[i].y + monitors[i].height) ? (wy + wh) : (monitors[i].y + monitors[i].height);
        int32_t area = (ox2 > ox1 && oy2 > oy1) ? (ox2 - ox1) * (oy2 - oy1) : 0;
        if (area > best_area) {
            best_area = area;
            best = i;
        }
    }
    return best;
}

// Snap scale to the nearest 0.25 increment and clamp to >= 1.0.
// This avoids non-uniform strokes and blurry rendering at fractional scales like 1.77x.
static float snap_scale(float scale) {
    int32_t quarters = (int32_t)(scale * 4.0f + 0.5f);
    if (quarters < 4) quarters = 4;  // minimum 1.0
    return (float)quarters / 4.0f;
}

// Compute the display scale factor for the given monitor index.
// Xft.dpi (user-configured) takes priority over RANDR hardware DPI.
static float compute_scale(const x11_connection_t* conn, x11_monitor_t* monitors, int32_t monitor_count,
                            int32_t idx) {
    float raw;
    if (conn->xft_dpi > 0.0f) {
        raw = conn->xft_dpi / 96.0f;
    } else if (monitor_count > 0) {
        raw = monitors[idx].scale;
    } else {
        raw = 1.0f;
    }
    return snap_scale(raw);
}

int main(void) {
    signal(SIGPIPE, SIG_IGN);

    x11_connection_t conn;
    if (!x11_connect(&conn)) {
        fprintf(stderr, "Can't connect to X11 display\n");
        return EXIT_FAILURE;
    }
    printf("Screen: %dx%d, MIT-SHM: %s, RANDR: %s (v%d.%d), SYNC: %s\n", conn.screen.width_in_pixels,
           conn.screen.height_in_pixels, conn.has_shm ? "yes" : "no", conn.has_randr ? "yes" : "no", conn.randr_major,
           conn.randr_minor, conn.has_sync ? "yes" : "no");

    // Validate root visual pixel format (expect standard RGB: R=0xFF0000 G=0xFF00 B=0xFF)
    if (conn.root_visual_red_mask != 0xFF0000 || conn.root_visual_green_mask != 0xFF00 ||
        conn.root_visual_blue_mask != 0xFF) {
        fprintf(stderr,
                "Warning: unexpected root visual masks R=0x%06X G=0x%06X B=0x%06X; "
                "colors may be wrong\n",
                conn.root_visual_red_mask, conn.root_visual_green_mask, conn.root_visual_blue_mask);
    }

    if (conn.xft_dpi > 0.0f)
        printf("Xft.dpi: %.0f (scale %.2f)\n", (double)conn.xft_dpi, (double)snap_scale(conn.xft_dpi / 96.0f));

    // Query monitors; keep the array alive for DPI-change detection
    x11_monitor_t* monitors = NULL;
    int32_t monitor_count = 0;
    int32_t primary_x = 0, primary_y = 0;
    int32_t primary_w = conn.screen.width_in_pixels, primary_h = conn.screen.height_in_pixels;
    int32_t primary_idx = 0;

    if (x11_randr_get_monitors(&conn, &monitors, &monitor_count) && monitor_count > 0) {
        printf("Monitors (%d):\n", monitor_count);
        for (int32_t i = 0; i < monitor_count; i++) {
            printf("  Monitor %d: %s %dx%d at (%d,%d) %.0f DPI (scale %.2f)%s\n", i + 1, monitors[i].name,
                   monitors[i].width, monitors[i].height, monitors[i].x, monitors[i].y, (double)monitors[i].dpi,
                   (double)monitors[i].scale, monitors[i].primary ? " [primary]" : "");
        }

        for (int32_t i = 0; i < monitor_count; i++) {
            if (monitors[i].primary) {
                primary_idx = i;
                break;
            }
        }
        primary_x = monitors[primary_idx].x;
        primary_y = monitors[primary_idx].y;
        primary_w = monitors[primary_idx].width;
        primary_h = monitors[primary_idx].height;
    }

    float scale = compute_scale(&conn, monitors, monitor_count, primary_idx);

    // Logical dimensions (design units) — updated on user resize
    int32_t logical_w = 640;
    int32_t logical_h = 480;

    int32_t window_width = (int32_t)(logical_w * scale);
    int32_t window_height = (int32_t)(logical_h * scale);
    int32_t window_x = primary_x + (primary_w - window_width) / 2;
    int32_t window_y = primary_y + (primary_h - window_height) / 2;

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

    // Set _NET_WM_WINDOW_TYPE before mapping so the compositor applies correct policies
    x11_change_property(&conn, X11_PROP_MODE_REPLACE, window, conn.net_wm_window_type, X11_ATOM_ATOM, 32,
                        &conn.net_wm_window_type_normal, sizeof(conn.net_wm_window_type_normal));

    // Create a SYNC counter and advertise it so compositors do tear-free resize
    uint32_t sync_counter = 0;
    if (conn.has_sync) {
        sync_counter = x11_sync_create_counter(&conn);
        x11_change_property(&conn, X11_PROP_MODE_REPLACE, window, conn.net_wm_sync_request_counter,
                            X11_ATOM_CARDINAL, 32, &sync_counter, sizeof(sync_counter));
    }

    x11_map_window(&conn, window);

    // Subscribe to screen-layout change events (monitor hot-plug, DPI changes)
    x11_randr_select_input(&conn, window);

    // Create image and canvas backed by it
    x11_image_t img;
    if (!x11_create_image(&conn, &img, window, window_width, window_height)) {
        fprintf(stderr, "Can't create image\n");
        if (sync_counter) x11_sync_destroy_counter(&conn, sync_counter);
        x11_disconnect(&conn);
        return EXIT_FAILURE;
    }

    canvas_t canvas;
    canvas_init(&canvas, logical_w, logical_h, img.pixels, scale);

    // Event loop
    x11_event_t event;
    bool running = true;
    bool has_pending_sync = false;
    int32_t pending_sync_lo = 0, pending_sync_hi = 0;
    while (running && x11_wait_for_event(&conn, &event)) {
        if (event.type == X11_CLIENT_MESSAGE_SYNC_REQUEST) {
            has_pending_sync = true;
            pending_sync_lo = event.sync_value_lo;
            pending_sync_hi = event.sync_value_hi;
        }

        if (event.type == X11_CONFIGURE_NOTIFY) {
            int32_t new_x = event.configure_x;
            int32_t new_y = event.configure_y;
            int32_t new_w = (int32_t)event.configure_width;
            int32_t new_h = (int32_t)event.configure_height;
            if (new_w <= 0 || new_h <= 0)
                continue;

            window_x = new_x;
            window_y = new_y;

            // Determine the scale for the monitor the window overlaps the most
            int32_t idx = (monitor_count > 0)
                              ? find_monitor_for_window(monitors, monitor_count, window_x, window_y, new_w, new_h)
                              : 0;
            float new_scale = compute_scale(&conn, monitors, monitor_count, idx);

            if (new_scale != scale) {
                // Monitor DPI changed: resize window to match new scale.
                // The resulting ConfigureNotify will carry the new physical size
                // and trigger the render path below.
                scale = new_scale;
                int32_t phys_w = (int32_t)(logical_w * scale);
                int32_t phys_h = (int32_t)(logical_h * scale);
                uint32_t size_vals[] = {(uint32_t)phys_w, (uint32_t)phys_h};
                x11_configure_window(&conn, window, X11_CONFIG_WINDOW_WIDTH | X11_CONFIG_WINDOW_HEIGHT, size_vals,
                                     sizeof(size_vals));
            } else if (new_w != img.width || new_h != img.height) {
                // User resize: derive logical dimensions from the new physical size.
                logical_w = (int32_t)(new_w / scale);
                logical_h = (int32_t)(new_h / scale);
                if (!x11_resize_image(&conn, &img, new_w, new_h)) {
                    fprintf(stderr, "Can't resize image\n");
                    running = false;
                    break;
                }
                canvas_init(&canvas, logical_w, logical_h, img.pixels, scale);
                render(&canvas);
                x11_put_image(&conn, window, &img);
            }

            // Acknowledge any pending WM sync request after processing this configure.
            // This allows the compositor to complete the resize without tearing.
            if (has_pending_sync && sync_counter) {
                x11_sync_set_counter(&conn, sync_counter, pending_sync_lo, pending_sync_hi);
                has_pending_sync = false;
            }
        }

        // Expose (count == 0 means no more expose events pending): render and blit.
        if (event.type == X11_EXPOSE && event.expose_count == 0) {
            render(&canvas);
            x11_put_image(&conn, window, &img);
        }

        if (event.type == X11_RANDR_SCREEN_CHANGE_NOTIFY) {
            // Monitor layout changed (hot-plug, resolution change, etc.).
            // Refresh the monitor cache and re-check DPI for the current window.
            x11_randr_free_monitors(monitors);
            monitors = NULL;
            monitor_count = 0;
            x11_randr_get_monitors(&conn, &monitors, &monitor_count);

            int32_t idx = (monitor_count > 0)
                              ? find_monitor_for_window(monitors, monitor_count, window_x, window_y, img.width,
                                                        img.height)
                              : 0;
            float new_scale = compute_scale(&conn, monitors, monitor_count, idx);
            if (new_scale != scale) {
                scale = new_scale;
                int32_t phys_w = (int32_t)(logical_w * scale);
                int32_t phys_h = (int32_t)(logical_h * scale);
                if (!x11_resize_image(&conn, &img, phys_w, phys_h)) {
                    fprintf(stderr, "Can't resize image\n");
                    running = false;
                    break;
                }
                uint32_t size_vals[] = {(uint32_t)phys_w, (uint32_t)phys_h};
                x11_configure_window(&conn, window, X11_CONFIG_WINDOW_WIDTH | X11_CONFIG_WINDOW_HEIGHT, size_vals,
                                     sizeof(size_vals));
                canvas_init(&canvas, logical_w, logical_h, img.pixels, scale);
                render(&canvas);
                x11_put_image(&conn, window, &img);
            }
        }

        if (event.type == X11_CLIENT_MESSAGE_CLOSE) {
            running = false;
        }
    }

    if (sync_counter) x11_sync_destroy_counter(&conn, sync_counter);
    x11_destroy_image(&conn, &img);
    x11_randr_free_monitors(monitors);
    x11_disconnect(&conn);
    return EXIT_SUCCESS;
}
