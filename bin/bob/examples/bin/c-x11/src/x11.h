/*
 * Copyright (c) 2022-2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

#pragma once

#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

#define X11_CREATE_WINDOW 1
#define X11_MAP_WINDOW 8
#define X11_CONFIGURE_WINDOW 12
#define X11_CHANGE_PROPERTY 18
#define X11_OPEN_FONT 45
#define X11_CLOSE_FONT 46
#define X11_CREATE_GC 55
#define X11_POLY_RECTANGLE 67
#define X11_IMAGE_TEXT_8 76

#define X11_EXPOSE 12

#define X11_GC_FOREGROUND 4
#define X11_GC_BACKGROUND 8
#define X11_GC_FONT 16384

#define X11_COPY_FROM_PARENT 0
#define X11_WINDOW_CLASS_INPUT_OUTPUT 1
#define X11_CW_BACK_PIXEL 2
#define X11_CW_EVENT_MASK 2048
#define X11_EVENT_MASK_EXPOSURE 32768

#define X11_PROP_MODE_REPLACE 0
#define X11_ATOM_STRING 31
#define X11_ATOM_WM_NAME 39

#define X11_CONFIG_WINDOW_X 1
#define X11_CONFIG_WINDOW_Y 2
#define X11_CONFIG_WINDOW_WIDTH 4
#define X11_CONFIG_WINDOW_HEIGHT 8

// X11 requests and responses
typedef struct x11_setup_request_t {
    uint8_t byte_order;
    uint8_t pad0;
    uint16_t protocol_major_version;
    uint16_t protocol_minor_version;
    uint16_t authorization_protocol_name_len;
    uint16_t authorization_protocol_data_len;
    uint8_t pad1[2];
} x11_setup_request_t;

typedef struct x11_setup_t {
    uint8_t status;
    uint8_t pad0;
    uint16_t protocol_major_version;
    uint16_t protocol_minor_version;
    uint16_t length;
    uint32_t release_number;
    uint32_t resource_id_base;
    uint32_t resource_id_mask;
    uint32_t motion_buffer_size;
    uint16_t vendor_len;
    uint16_t maximum_request_length;
    uint8_t roots_len;
    uint8_t pixmap_formats_len;
    uint8_t image_byte_order;
    uint8_t bitmap_format_bit_order;
    uint8_t bitmap_format_scanline_unit;
    uint8_t bitmap_format_scanline_pad;
    uint8_t min_keycode;
    uint8_t max_keycode;
    uint8_t pad1[4];
} x11_setup_t;
typedef struct x11_format_t {
    uint8_t depth;
    uint8_t bits_per_pixel;
    uint8_t scanline_pad;
    uint8_t pad0[5];
} x11_format_t;
typedef struct x11_screen_t {
    uint32_t root;
    uint32_t default_colormap;
    uint32_t white_pixel;
    uint32_t black_pixel;
    uint32_t current_input_masks;
    uint16_t width_in_pixels;
    uint16_t height_in_pixels;
    uint16_t width_in_millimeters;
    uint16_t height_in_millimeters;
    uint16_t min_installed_maps;
    uint16_t max_installed_maps;
    uint32_t root_visual;
    uint8_t backing_stores;
    uint8_t save_unders;
    uint8_t root_depth;
    uint8_t allowed_depths_len;
} x11_screen_t;
typedef struct x11_depth_t {
    uint8_t depth;
    uint8_t pad0;
    uint16_t visuals_len;
    uint8_t pad1[4];
} x11_depth_t;
typedef struct x11_visualtype_t {
    uint32_t visual_id;
    uint8_t _class;
    uint8_t bits_per_rgb_value;
    uint16_t colormap_entries;
    uint32_t red_mask;
    uint32_t green_mask;
    uint32_t blue_mask;
    uint8_t pad0[4];
} x11_visualtype_t;

typedef struct x11_open_font_request_t {
    uint8_t major_opcode;
    uint8_t pad0;
    uint16_t length;
    uint32_t fid;
    uint16_t name_len;
    uint8_t pad1[2];
} x11_open_font_request_t;

typedef struct x11_close_font_request_t {
    uint8_t major_opcode;
    uint8_t pad0;
    uint16_t length;
    uint32_t font;
} x11_close_font_request_t;

typedef struct x11_create_gc_request_t {
    uint8_t major_opcode;
    uint8_t pad0;
    uint16_t length;
    uint32_t cid;
    uint32_t drawable;
    uint32_t value_mask;
} x11_create_gc_request_t;

typedef struct x11_create_window_request_t {
    uint8_t major_opcode;
    uint8_t depth;
    uint16_t length;
    uint32_t wid;
    uint32_t parent;
    int16_t x;
    int16_t y;
    uint16_t width;
    uint16_t height;
    uint16_t border_width;
    uint16_t _class;
    uint32_t visual;
    uint32_t value_mask;
} x11_create_window_request_t;

typedef struct x11_change_property_request_t {
    uint8_t major_opcode;
    uint8_t mode;
    uint16_t length;
    uint32_t window;
    uint32_t property;
    uint32_t type;
    uint8_t format;
    uint8_t pad0[3];
    uint32_t data_len;
} x11_change_property_request_t;

typedef struct x11_configure_window_request_t {
    uint8_t major_opcode;
    uint8_t pad0;
    uint16_t length;
    uint32_t window;
    uint16_t value_mask;
    uint8_t pad1[2];
} x11_configure_window_request_t;

typedef struct x11_map_window_request_t {
    uint8_t major_opcode;
    uint8_t pad0;
    uint16_t length;
    uint32_t window;
} x11_map_window_request_t;

typedef struct x11_poly_rectangle_request_t {
    uint8_t major_opcode;
    uint8_t pad0;
    uint16_t length;
    uint32_t drawable;
    uint32_t gc;
} x11_poly_rectangle_request_t;
typedef struct x11_rectangle_t {
    int16_t x;
    int16_t y;
    uint16_t width;
    uint16_t height;
} x11_rectangle_t;

typedef struct x11_image_text_8_request_t {
    uint8_t major_opcode;
    uint8_t string_len;
    uint16_t length;
    uint32_t drawable;
    uint32_t gc;
    int16_t x;
    int16_t y;
} x11_image_text_8_request_t;

// X11 structs
typedef struct x11_connection_t {
    int32_t fd;
    uint32_t id;
    uint32_t id_inc;
    x11_screen_t screen;
} x11_connection_t;

typedef struct x11_event_t {
    uint8_t type;
} x11_event_t;

bool x11_connect(x11_connection_t* conn);

uint32_t x11_generate_id(x11_connection_t* conn);

void x11_open_font(x11_connection_t* conn, uint32_t fid, char* name, size_t name_size);

void x11_close_font(x11_connection_t* conn, uint32_t font);

void x11_create_gc(x11_connection_t* conn, uint32_t cid, uint32_t drawable, uint32_t value_mask, uint32_t* value_list,
                   size_t value_list_size);

void x11_create_window(x11_connection_t* conn, uint8_t depth, uint32_t wid, uint32_t parent, int16_t x, int16_t y,
                       uint16_t width, uint16_t height, uint16_t border_width, uint16_t _class, uint32_t visual,
                       uint32_t value_mask, uint32_t* value_list, size_t value_list_size);

void x11_change_property(x11_connection_t* conn, uint8_t mode, uint32_t window, uint32_t property, uint32_t type,
                         uint8_t format, void* data, size_t data_size);

void x11_configure_window(x11_connection_t* conn, uint32_t window, uint32_t value_mask, uint32_t* value_list,
                          size_t value_list_size);

void x11_map_window(x11_connection_t* conn, uint32_t window);

void x11_poly_rectangle(x11_connection_t* conn, uint32_t drawable, uint32_t gc, x11_rectangle_t* rectangles,
                        size_t rectangles_size);

void x11_image_text_8(x11_connection_t* conn, uint32_t drawable, uint32_t gc, int16_t x, int16_t y, const char* string,
                      size_t string_size);

bool x11_wait_for_event(x11_connection_t* conn, x11_event_t* event);

void x11_disconnect(x11_connection_t* conn);
