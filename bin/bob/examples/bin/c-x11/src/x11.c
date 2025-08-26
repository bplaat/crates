/*
 * Copyright (c) 2022-2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

#include "x11.h"

#include <arpa/inet.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/socket.h>
#include <sys/un.h>
#include <unistd.h>

typedef struct {
    uint16_t name_len;
    char* name;
    uint16_t data_len;
    uint8_t* data;
} x11_cookie_t;

x11_cookie_t* parse_xauthority(const char* xauth_path) {
    FILE* xauth_file = fopen(xauth_path, "rb");
    if (xauth_file) {
        while (1) {
            uint16_t family, address_len, display_len, local_name_len, local_data_len;
            if (fread(&family, 2, 1, xauth_file) != 1) break;
            family = ntohs(family);

            if (fread(&address_len, 2, 1, xauth_file) != 1) break;
            address_len = ntohs(address_len);
            if (fseek(xauth_file, address_len, SEEK_CUR) != 0) break;

            if (fread(&display_len, 2, 1, xauth_file) != 1) break;
            display_len = ntohs(display_len);
            if (fseek(xauth_file, display_len, SEEK_CUR) != 0) break;

            if (fread(&local_name_len, 2, 1, xauth_file) != 1) break;
            local_name_len = ntohs(local_name_len);
            char local_name[local_name_len];
            if (fread(local_name, 1, local_name_len, xauth_file) != local_name_len) break;

            if (fread(&local_data_len, 2, 1, xauth_file) != 1) break;
            local_data_len = ntohs(local_data_len);
            uint8_t local_data[local_data_len];
            if (fread(local_data, 1, local_data_len, xauth_file) != local_data_len) break;

            if (local_name_len == 18 && memcmp(local_name, "MIT-MAGIC-COOKIE-1", 18) == 0) {
                x11_cookie_t* cookie = malloc(sizeof(x11_cookie_t));
                cookie->name_len = local_name_len;
                cookie->name = malloc(local_name_len);
                memcpy(cookie->name, local_name, local_name_len);
                cookie->data_len = local_data_len;
                cookie->data = malloc(local_data_len);
                memcpy(cookie->data, local_data, local_data_len);
                return cookie;
            }
        }
        fclose(xauth_file);
    }
    return NULL;
}

bool x11_connect(x11_connection_t* conn) {
    // Get DISPLAY env variable
    char display[512] = ":0";
    if (getenv("DISPLAY")) {
        strcpy(display, getenv("DISPLAY"));
    }
    if (display[0] == ':') {
        int32_t display_number = strtol(&display[1], NULL, 10);
        sprintf(display, "/tmp/.X11-unix/X%d", display_number);
    }

    // Try to read and parse .Xauthority
    char xauth_path[512];
    sprintf(xauth_path, "%s/.Xauthority", getenv("HOME"));
    if (getenv("XAUTHORITY")) {
        strcpy(xauth_path, getenv("XAUTHORITY"));
    }
    x11_cookie_t* cookie = parse_xauthority(xauth_path);

    // Connect to X11 server
    conn->fd = socket(AF_UNIX, SOCK_STREAM, 0);
    struct sockaddr_un saddr = {.sun_family = AF_UNIX};
    strcpy(saddr.sun_path, display);
    connect(conn->fd, (struct sockaddr*)&saddr, sizeof(saddr));

    // Send setup request message
    x11_setup_request_t setup_request = {
#if __BYTE_ORDER__ == __ORDER_BIG_ENDIAN__
        .byte_order = 'B',
#else
        .byte_order = 'l',
#endif
        .protocol_major_version = 11,
        .protocol_minor_version = 0,
        .authorization_protocol_name_len = cookie ? cookie->name_len : 0,
        .authorization_protocol_data_len = cookie ? cookie->data_len : 0,
    };
    write(conn->fd, &setup_request, sizeof(x11_setup_request_t));
    if (cookie) {
        uint8_t pad[4] = {0};
        write(conn->fd, cookie->name, cookie->name_len);
        if (cookie->name_len % 4 != 0) write(conn->fd, pad, 4 - (cookie->name_len % 4));
        write(conn->fd, cookie->data, cookie->data_len);
        if (cookie->data_len % 4 != 0) write(conn->fd, pad, 4 - (cookie->data_len % 4));

        free(cookie->name);
        free(cookie->data);
        free(cookie);
    }

    // Read setup
    x11_setup_t setup;
    read(conn->fd, &setup, sizeof(setup));
    if (setup.status != 1) {
        return false;
    }
    conn->id = setup.resource_id_base;
    conn->id_inc = setup.resource_id_mask & -(setup.resource_id_mask);

    // Read vendor
    char unused_vendor[setup.vendor_len];
    read(conn->fd, &unused_vendor, setup.vendor_len);

    // Read formats
    x11_format_t unused_formats[setup.pixmap_formats_len];
    read(conn->fd, &unused_formats, setup.pixmap_formats_len * sizeof(x11_format_t));

    // Read screens
    x11_screen_t unused_screen;
    for (int32_t i = 0; i < setup.roots_len; i++) {
        x11_screen_t* screen = i == 0 ? &conn->screen : &unused_screen;
        read(conn->fd, screen, sizeof(x11_screen_t));

        for (int32_t j = 0; j < screen->allowed_depths_len; j++) {
            x11_depth_t unused_depth;
            read(conn->fd, &unused_depth, sizeof(x11_depth_t));

            for (int32_t k = 0; k < unused_depth.visuals_len; k++) {
                x11_visualtype_t unused_visualtype;
                read(conn->fd, &unused_visualtype, sizeof(x11_visualtype_t));
            }
        }
    }
    return true;
}

uint32_t x11_generate_id(x11_connection_t* conn) {
    uint32_t id = conn->id;
    conn->id += conn->id_inc;
    return id;
}

void x11_open_font(x11_connection_t* conn, uint32_t fid, char* name, size_t name_size) {
    size_t aligned_name_size = (name_size + 3) & ~3;
    x11_open_font_request_t open_font_request = {
        .major_opcode = X11_OPEN_FONT,
        .length = (sizeof(x11_open_font_request_t) + aligned_name_size) / 4,
        .fid = fid,
        .name_len = name_size,
    };
    write(conn->fd, &open_font_request, sizeof(x11_open_font_request_t));
    write(conn->fd, name, aligned_name_size);
}

void x11_close_font(x11_connection_t* conn, uint32_t font) {
    x11_close_font_request_t close_font_request = {
        .major_opcode = X11_CLOSE_FONT, .length = (sizeof(x11_close_font_request_t)) / 4, .font = font};
    write(conn->fd, &close_font_request, sizeof(x11_close_font_request_t));
}

void x11_create_gc(x11_connection_t* conn, uint32_t cid, uint32_t drawable, uint32_t value_mask, uint32_t* value_list,
                   size_t value_list_size) {
    x11_create_gc_request_t create_gc_request = {.major_opcode = X11_CREATE_GC,
                                                 .length = (sizeof(x11_create_gc_request_t) + value_list_size) / 4,
                                                 .cid = cid,
                                                 .drawable = drawable,
                                                 .value_mask = value_mask};
    write(conn->fd, &create_gc_request, sizeof(x11_create_gc_request_t));
    write(conn->fd, value_list, value_list_size);
}

void x11_create_window(x11_connection_t* conn, uint8_t depth, uint32_t wid, uint32_t parent, int16_t x, int16_t y,
                       uint16_t width, uint16_t height, uint16_t border_width, uint16_t _class, uint32_t visual,
                       uint32_t value_mask, uint32_t* value_list, size_t value_list_size) {
    x11_create_window_request_t create_window_request = {
        .major_opcode = X11_CREATE_WINDOW,
        .depth = depth,
        .length = (sizeof(x11_create_window_request_t) + value_list_size) / 4,
        .wid = wid,
        .parent = parent,
        .x = x,
        .y = y,
        .width = width,
        .height = height,
        .border_width = border_width,
        ._class = _class,
        .visual = visual,
        .value_mask = value_mask};
    write(conn->fd, &create_window_request, sizeof(x11_create_window_request_t));
    write(conn->fd, value_list, value_list_size);
}

void x11_change_property(x11_connection_t* conn, uint8_t mode, uint32_t window, uint32_t property, uint32_t type,
                         uint8_t format, void* data, size_t data_size) {
    size_t aligned_data_size = (data_size + 3) & ~3;
    x11_change_property_request_t change_property_request = {
        .major_opcode = X11_CHANGE_PROPERTY,
        .mode = mode,
        .length = (sizeof(x11_change_property_request_t) + aligned_data_size) / 4,
        .window = window,
        .property = property,
        .type = type,
        .format = format,
        .data_len = data_size};
    write(conn->fd, &change_property_request, sizeof(x11_change_property_request_t));
    write(conn->fd, data, aligned_data_size);
}

void x11_configure_window(x11_connection_t* conn, uint32_t window, uint32_t value_mask, uint32_t* value_list,
                          size_t value_list_size) {
    x11_configure_window_request_t configure_window_request = {
        .major_opcode = X11_CONFIGURE_WINDOW,
        .length = (sizeof(x11_configure_window_request_t) + value_list_size) / 4,
        .window = window,
        .value_mask = value_mask};
    write(conn->fd, &configure_window_request, sizeof(x11_configure_window_request_t));
    write(conn->fd, value_list, value_list_size);
}

void x11_map_window(x11_connection_t* conn, uint32_t window) {
    x11_map_window_request_t map_window_request = {
        .major_opcode = X11_MAP_WINDOW, .length = sizeof(x11_map_window_request_t) / 4, .window = window};
    write(conn->fd, &map_window_request, sizeof(x11_map_window_request_t));
}

void x11_poly_rectangle(x11_connection_t* conn, uint32_t drawable, uint32_t gc, x11_rectangle_t* rectangles,
                        size_t rectangles_size) {
    x11_poly_rectangle_request_t poly_rectangle_request = {
        .major_opcode = X11_POLY_RECTANGLE,
        .length = (sizeof(x11_poly_rectangle_request_t) + rectangles_size) / 4,
        .drawable = drawable,
        .gc = gc};
    write(conn->fd, &poly_rectangle_request, sizeof(x11_poly_rectangle_request_t));
    write(conn->fd, rectangles, rectangles_size);
}

void x11_image_text_8(x11_connection_t* conn, uint32_t drawable, uint32_t gc, int16_t x, int16_t y, const char* string,
                      size_t string_size) {
    size_t aligned_string_size = (string_size + 3) & ~3;
    x11_image_text_8_request_t image_text_8_request = {
        .major_opcode = X11_IMAGE_TEXT_8,
        .string_len = string_size,
        .length = (sizeof(x11_image_text_8_request_t) + aligned_string_size) / 4,
        .drawable = drawable,
        .gc = gc,
        .x = x,
        .y = y,
    };
    write(conn->fd, &image_text_8_request, sizeof(x11_image_text_8_request_t));
    write(conn->fd, string, aligned_string_size);
}

bool x11_wait_for_event(x11_connection_t* conn, x11_event_t* event) {
    if (read(conn->fd, &event->type, 1) == 0) {
        return false;
    }
    event->type &= ~0x80;
    if (event->type == X11_EXPOSE) {
        uint8_t unused[13];
        read(conn->fd, unused, sizeof(unused));
        return true;
    }
    // TODO: parse unknown messages
    return true;
}

void x11_disconnect(x11_connection_t* conn) {
    close(conn->fd);
}
