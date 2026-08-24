/*
 * Copyright (c) 2022-2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

// A pure C X11 software-framebuffer example with a time-based game loop.

#define _POSIX_C_SOURCE 200112L

#include <signal.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>
#include <unistd.h>

#include "canvas.h"
#include "x11.h"

static void render(canvas_t* canvas, double time_seconds) {
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

    // A time-based animation makes it clear that rendering is independent of
    // X11 event delivery. Use design units so it stays correct at every DPI.
    float travel = (float)canvas->width - 80.0f;
    float phase = (float)(time_seconds * 120.0);
    int32_t period = travel > 1.0f ? (int32_t)(travel * 2.0f) : 1;
    float x = (float)((int32_t)phase % period);
    if (x > travel)
        x = (float)period - x;
    canvas_fill_rect(canvas, x, (float)canvas->height - 48.0f, 80.0f, 24.0f, CANVAS_COLOR(32, 32, 32));
}

static int64_t monotonic_ns(void) {
    struct timespec now;
    if (clock_gettime(CLOCK_MONOTONIC, &now) != 0)
        return 0;
    return (int64_t)now.tv_sec * 1000000000LL + now.tv_nsec;
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
    if (quarters < 4)
        quarters = 4;  // minimum 1.0
    return (float)quarters / 4.0f;
}

// Compute the display scale factor for the given monitor index.
// Xft.dpi (user-configured) takes priority over RANDR hardware DPI.
static float compute_scale(const x11_connection_t* conn, x11_monitor_t* monitors, int32_t monitor_count, int32_t idx) {
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
    printf("Screen: %dx%d, MIT-SHM: %s, RANDR: %s (v%d.%d), SYNC: %s, XDBE: %s\n", conn.screen.width_in_pixels,
           conn.screen.height_in_pixels, conn.has_shm ? "yes" : "no", conn.has_randr ? "yes" : "no", conn.randr_major,
           conn.randr_minor, conn.has_sync ? "yes" : "no", conn.has_xdbe ? "yes" : "no");

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

    // Logical dimensions (design units) - updated on user resize
    int32_t logical_w = 640;
    int32_t logical_h = 480;

    int32_t window_width = (int32_t)(logical_w * scale);
    int32_t window_height = (int32_t)(logical_h * scale);
    int32_t window_x = primary_x + (primary_w - window_width) / 2;
    int32_t window_y = primary_y + (primary_h - window_height) / 2;

    const char* window_title = "Hello Canvas!";
    uint32_t window = x11_generate_id(&conn);
    // BackPixel = white: X server fills newly-exposed areas with white on Expose
    // and after unmap/remap (e.g., unminimize). When XDBE is active, the back
    // buffer covers all rendering so this only matters for the brief moment before
    // the first frame is painted. Without XDBE, it prevents black flashing on resize.
    uint32_t create_window_list[] = {conn.screen.white_pixel /* BackPixel */,
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
        x11_change_property(&conn, X11_PROP_MODE_REPLACE, window, conn.net_wm_sync_request_counter, X11_ATOM_CARDINAL,
                            32, &sync_counter, sizeof(sync_counter));
    }

    // Create image and canvas backed by it
    x11_image_t img;
    if (!x11_create_image(&conn, &img, window, window_width, window_height)) {
        fprintf(stderr, "Can't create image\n");
        if (sync_counter)
            x11_sync_destroy_counter(&conn, sync_counter);
        x11_disconnect(&conn);
        return EXIT_FAILURE;
    }

    // XDBE is useful on traditional Xorg, but XQuartz implements it through an
    // additional full-window copy that makes interactive resizing much slower.
    // The native macOS window is already composited, so direct presentation is
    // both faster and tear-free there.
    bool use_xdbe = conn.has_xdbe;
#ifdef __APPLE__
    use_xdbe = false;
#endif
    uint32_t back_buffer = use_xdbe ? x11_xdbe_alloc_back_buffer(&conn, window) : 0;
    uint32_t render_target = back_buffer ? back_buffer : window;

    canvas_t canvas;
    canvas_init(&canvas, logical_w, logical_h, img.width, img.height, img.pixels, scale);

    // Finish all synchronous capability checks before mapping. Once mapped,
    // ordinary window events may be interleaved with round-trip replies.
    x11_randr_select_input(&conn, window);
    x11_map_window(&conn, window);

    // Game loop. poll() sleeps until either X11 has work or the next frame is
    // due, so the loop is responsive without spinning. Simulation is based on
    // monotonic time and is therefore independent of rendering speed.
    x11_event_t event;
    bool running = true;
    bool needs_render = true;
    bool frame_in_flight = false;
    int32_t pending_resize_w = 0;
    int32_t pending_resize_h = 0;
    bool has_pending_sync = false;
    int32_t pending_sync_lo = 0, pending_sync_hi = 0;
    const int64_t frame_ns = 1000000000LL / 60;
    int64_t start_ns = monotonic_ns();
    int64_t next_frame_ns = start_ns;
    while (running) {
        int64_t now_ns = monotonic_ns();
        int64_t wait_ns = next_frame_ns - now_ns;
        int32_t timeout_ms = wait_ns > 0 ? (int32_t)((wait_ns + 999999) / 1000000) : 0;
        int poll_result = x11_poll_event(&conn, &event, timeout_ms);
        if (poll_result < 0)
            break;
        if (poll_result == 0)
            event.type = 0;

        if (event.type == X11_SHM_COMPLETION)
            frame_in_flight = false;

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
            if (new_w <= 0 || new_h <= 0) {
                // Zero-size configure (e.g. minimize): ack any pending sync so
                // the compositor does not stall waiting for the client to reply.
                if (has_pending_sync && sync_counter) {
                    x11_sync_set_counter(&conn, sync_counter, pending_sync_lo, pending_sync_hi);
                    has_pending_sync = false;
                }
                continue;
            }

            // Only synthetic ConfigureNotify (WM via SendEvent) carries root-relative coords.
            if (event.configure_is_synthetic) {
                window_x = new_x;
                window_y = new_y;
            }

            // Determine the scale for the monitor the window overlaps the most
            int32_t idx = (monitor_count > 0)
                              ? find_monitor_for_window(monitors, monitor_count, window_x, window_y, new_w, new_h)
                              : 0;
            float new_scale = compute_scale(&conn, monitors, monitor_count, idx);

            if (new_scale != scale) {
                // DPI changed: send ConfigureWindow to resize; the WM will send a
                // fresh _NET_WM_SYNC_REQUEST before the follow-up ConfigureNotify.
                // Do not ack the current sync here -- no frame has been rendered.
                scale = new_scale;
                int32_t phys_w = (int32_t)(logical_w * scale);
                int32_t phys_h = (int32_t)(logical_h * scale);
                uint32_t size_vals[] = {(uint32_t)phys_w, (uint32_t)phys_h};
                x11_configure_window(&conn, window, X11_CONFIG_WINDOW_WIDTH | X11_CONFIG_WINDOW_HEIGHT, size_vals,
                                     sizeof(size_vals));
            } else if (new_w != img.width || new_h != img.height) {
                // MIT-SHM ownership remains with the server until Completion.
                // Defer reallocating or drawing into that memory until then.
                if (frame_in_flight) {
                    pending_resize_w = new_w;
                    pending_resize_h = new_h;
                    continue;
                }
                // User resize: derive logical dimensions from the new physical size.
                logical_w = (int32_t)(new_w / scale);
                logical_h = (int32_t)(new_h / scale);
                if (!x11_resize_image(&conn, &img, new_w, new_h)) {
                    fprintf(stderr, "Can't resize image\n");
                    running = false;
                    break;
                }
                canvas_init(&canvas, logical_w, logical_h, img.width, img.height, img.pixels, scale);
                frame_in_flight = false;
                needs_render = true;
                if (has_pending_sync && sync_counter) {
                    x11_sync_set_counter(&conn, sync_counter, pending_sync_lo, pending_sync_hi);
                    has_pending_sync = false;
                }
            } else if (has_pending_sync && sync_counter) {
                // Position-only change: no repaint needed, ack immediately.
                x11_sync_set_counter(&conn, sync_counter, pending_sync_lo, pending_sync_hi);
                has_pending_sync = false;
            }
        }

        // Expose (count == 0 means no more expose events pending): render and blit.
        if (event.type == X11_EXPOSE && event.expose_count == 0) {
            needs_render = true;
        }

        if (event.type == X11_RANDR_SCREEN_CHANGE_NOTIFY) {
            // Monitor layout changed (hot-plug, resolution change, etc.).
            // Refresh the monitor cache and re-check DPI for the current window.
            x11_randr_free_monitors(monitors);
            monitors = NULL;
            monitor_count = 0;
            x11_randr_get_monitors(&conn, &monitors, &monitor_count);

            int32_t idx = (monitor_count > 0) ? find_monitor_for_window(monitors, monitor_count, window_x, window_y,
                                                                        img.width, img.height)
                                              : 0;
            float new_scale = compute_scale(&conn, monitors, monitor_count, idx);
            if (new_scale != scale) {
                scale = new_scale;
                int32_t phys_w = (int32_t)(logical_w * scale);
                int32_t phys_h = (int32_t)(logical_h * scale);
                uint32_t size_vals[] = {(uint32_t)phys_w, (uint32_t)phys_h};
                x11_configure_window(&conn, window, X11_CONFIG_WINDOW_WIDTH | X11_CONFIG_WINDOW_HEIGHT, size_vals,
                                     sizeof(size_vals));
            }
        }

        if (event.type == X11_CLIENT_MESSAGE_CLOSE) {
            running = false;
        }

        if (!frame_in_flight && pending_resize_w > 0 && pending_resize_h > 0) {
            logical_w = (int32_t)(pending_resize_w / scale);
            logical_h = (int32_t)(pending_resize_h / scale);
            if (!x11_resize_image(&conn, &img, pending_resize_w, pending_resize_h)) {
                fprintf(stderr, "Can't resize image\n");
                running = false;
            } else {
                canvas_init(&canvas, logical_w, logical_h, img.width, img.height, img.pixels, scale);
                needs_render = true;
            }
            pending_resize_w = 0;
            pending_resize_h = 0;
        }

        now_ns = monotonic_ns();
        if (now_ns >= next_frame_ns)
            needs_render = true;
        // Drain queued ConfigureNotify events before drawing. This turns a burst
        // of live-resize sizes into one framebuffer resize and one final blit.
        if (needs_render && !frame_in_flight && !x11_has_event_pending(&conn) && running) {
            render(&canvas, (double)(now_ns - start_ns) / 1000000000.0);
            x11_put_image(&conn, render_target, &img);
            if (back_buffer)
                x11_xdbe_swap_buffers(&conn, window);
            frame_in_flight = conn.has_shm && img.shmseg != 0;
            needs_render = false;
            if (has_pending_sync && sync_counter) {
                x11_sync_set_counter(&conn, sync_counter, pending_sync_lo, pending_sync_hi);
                has_pending_sync = false;
            }
        }
        if (now_ns >= next_frame_ns) {
            next_frame_ns += frame_ns;
            if (next_frame_ns <= now_ns)
                next_frame_ns = now_ns + frame_ns;
        }
    }

    if (sync_counter)
        x11_sync_destroy_counter(&conn, sync_counter);
    if (back_buffer)
        x11_xdbe_free_back_buffer(&conn, back_buffer);
    x11_destroy_image(&conn, &img);
    x11_randr_free_monitors(monitors);
    x11_disconnect(&conn);
    return EXIT_SUCCESS;
}
