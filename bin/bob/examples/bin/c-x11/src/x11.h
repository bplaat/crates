/*
 * Copyright (c) 2022-2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

#pragma once

#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

// Packed attribute for wire-format structs
#if defined(__GNUC__) || defined(__clang__)
#define X11_PACKED __attribute__((packed))
#else
#define X11_PACKED
#endif

// Byte order detection
#if defined(__BYTE_ORDER__) && defined(__ORDER_BIG_ENDIAN__)
#if __BYTE_ORDER__ == __ORDER_BIG_ENDIAN__
#define X11_BIG_ENDIAN 1
#endif
#elif defined(__BIG_ENDIAN__) || defined(__ARMEB__) || defined(__MIPSEB__)
#define X11_BIG_ENDIAN 1
#endif

// Opcodes
#define X11_CREATE_WINDOW 1
#define X11_MAP_WINDOW 8
#define X11_CONFIGURE_WINDOW 12
#define X11_INTERN_ATOM 16
#define X11_GET_ATOM_NAME 17
#define X11_CHANGE_PROPERTY 18
#define X11_FREE_GC 60
#define X11_CREATE_GC 55
#define X11_PUT_IMAGE 72
#define X11_QUERY_EXTENSION 98

// RANDR minor opcodes
#define X11_RANDR_QUERY_VERSION 0
#define X11_RANDR_GET_SCREEN_RESOURCES 8
#define X11_RANDR_GET_CRTC_INFO 20
#define X11_RANDR_GET_MONITORS 42

// Event types
#define X11_ERROR 0
#define X11_EXPOSE 12
#define X11_CONFIGURE_NOTIFY 22
#define X11_CLIENT_MESSAGE 33
#define X11_CLIENT_MESSAGE_CLOSE 255

// GC value masks
#define X11_GC_GRAPHICS_EXPOSURES 65536

// Window constants
#define X11_COPY_FROM_PARENT 0
#define X11_WINDOW_CLASS_INPUT_OUTPUT 1
#define X11_CW_BACK_PIXMAP 1
#define X11_CW_BACK_PIXEL 2
#define X11_CW_EVENT_MASK 2048
#define X11_EVENT_MASK_EXPOSURE 32768
#define X11_EVENT_MASK_STRUCTURE_NOTIFY 131072

// PutImage format
#define X11_IMAGE_FORMAT_Z_PIXMAP 2

// MIT-SHM sub-opcodes
#define X11_SHM_ATTACH 1
#define X11_SHM_DETACH 2
#define X11_SHM_PUT_IMAGE 3

// Property constants
#define X11_PROP_MODE_REPLACE 0
#define X11_ATOM_ATOM 4
#define X11_ATOM_CARDINAL 6
#define X11_ATOM_STRING 31
#define X11_ATOM_WM_HINTS 35
#define X11_ATOM_WM_NAME 39
#define X11_ATOM_WM_NORMAL_HINTS 40
#define X11_ATOM_WM_SIZE_HINTS 41
#define X11_ATOM_WM_CLASS 67
#define X11_ATOM_WM_CLIENT_MACHINE 36

// Configure window masks
#define X11_CONFIG_WINDOW_X 1
#define X11_CONFIG_WINDOW_Y 2
#define X11_CONFIG_WINDOW_WIDTH 4
#define X11_CONFIG_WINDOW_HEIGHT 8

// X11 requests and responses
typedef struct X11_PACKED x11_setup_request_t {
    uint8_t byte_order;
    uint8_t pad0;
    uint16_t protocol_major_version;
    uint16_t protocol_minor_version;
    uint16_t authorization_protocol_name_len;
    uint16_t authorization_protocol_data_len;
    uint8_t pad1[2];
} x11_setup_request_t;

typedef struct X11_PACKED x11_setup_t {
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

typedef struct X11_PACKED x11_format_t {
    uint8_t depth;
    uint8_t bits_per_pixel;
    uint8_t scanline_pad;
    uint8_t pad0[5];
} x11_format_t;

typedef struct X11_PACKED x11_screen_t {
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

typedef struct X11_PACKED x11_depth_t {
    uint8_t depth;
    uint8_t pad0;
    uint16_t visuals_len;
    uint8_t pad1[4];
} x11_depth_t;

typedef struct X11_PACKED x11_visualtype_t {
    uint32_t visual_id;
    uint8_t _class;
    uint8_t bits_per_rgb_value;
    uint16_t colormap_entries;
    uint32_t red_mask;
    uint32_t green_mask;
    uint32_t blue_mask;
    uint8_t pad0[4];
} x11_visualtype_t;

typedef struct X11_PACKED x11_intern_atom_request_t {
    uint8_t major_opcode;
    uint8_t only_if_exists;
    uint16_t length;
    uint16_t name_len;
    uint8_t pad0[2];
} x11_intern_atom_request_t;

typedef struct X11_PACKED x11_intern_atom_reply_t {
    uint8_t reply;
    uint8_t pad0;
    uint16_t sequence_number;
    uint32_t reply_length;
    uint32_t atom;
    uint8_t pad1[20];
} x11_intern_atom_reply_t;

typedef struct X11_PACKED x11_create_window_request_t {
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

typedef struct X11_PACKED x11_change_property_request_t {
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

typedef struct X11_PACKED x11_configure_window_request_t {
    uint8_t major_opcode;
    uint8_t pad0;
    uint16_t length;
    uint32_t window;
    uint16_t value_mask;
    uint8_t pad1[2];
} x11_configure_window_request_t;

typedef struct X11_PACKED x11_map_window_request_t {
    uint8_t major_opcode;
    uint8_t pad0;
    uint16_t length;
    uint32_t window;
} x11_map_window_request_t;

typedef struct X11_PACKED x11_query_extension_request_t {
    uint8_t major_opcode;
    uint8_t pad0;
    uint16_t length;
    uint16_t name_len;
    uint8_t pad1[2];
} x11_query_extension_request_t;

typedef struct X11_PACKED x11_query_extension_reply_t {
    uint8_t reply;
    uint8_t pad0;
    uint16_t sequence_number;
    uint32_t reply_length;
    uint8_t present;
    uint8_t major_opcode;
    uint8_t first_event;
    uint8_t first_error;
    uint8_t pad1[20];
} x11_query_extension_reply_t;

typedef struct X11_PACKED x11_create_gc_request_t {
    uint8_t major_opcode;
    uint8_t pad0;
    uint16_t length;
    uint32_t cid;
    uint32_t drawable;
    uint32_t value_mask;
} x11_create_gc_request_t;

typedef struct X11_PACKED x11_free_gc_request_t {
    uint8_t major_opcode;
    uint8_t pad0;
    uint16_t length;
    uint32_t gc;
} x11_free_gc_request_t;

typedef struct X11_PACKED x11_big_req_enable_request_t {
    uint8_t major_opcode;
    uint8_t minor_opcode;
    uint16_t length;
} x11_big_req_enable_request_t;

typedef struct X11_PACKED x11_big_req_enable_reply_t {
    uint8_t reply;
    uint8_t pad0;
    uint16_t sequence_number;
    uint32_t reply_length;
    uint32_t maximum_request_length;
    uint8_t pad1[20];
} x11_big_req_enable_reply_t;

typedef struct X11_PACKED x11_put_image_big_request_t {
    uint8_t major_opcode;
    uint8_t format;
    uint16_t length;      // 0 = BigRequests marker
    uint32_t big_length;  // actual length in 4-byte units (includes this field)
    uint32_t drawable;
    uint32_t gc;
    uint16_t width;
    uint16_t height;
    int16_t dst_x;
    int16_t dst_y;
    uint8_t left_pad;
    uint8_t depth;
    uint8_t pad0[2];
} x11_put_image_big_request_t;

typedef struct X11_PACKED x11_put_image_request_t {
    uint8_t major_opcode;
    uint8_t format;
    uint16_t length;
    uint32_t drawable;
    uint32_t gc;
    uint16_t width;
    uint16_t height;
    int16_t dst_x;
    int16_t dst_y;
    uint8_t left_pad;
    uint8_t depth;
    uint8_t pad0[2];
} x11_put_image_request_t;

typedef struct X11_PACKED x11_shm_attach_request_t {
    uint8_t major_opcode;
    uint8_t minor_opcode;
    uint16_t length;
    uint32_t shmseg;
    uint32_t shmid;
    uint8_t read_only;
    uint8_t pad0[3];
} x11_shm_attach_request_t;

typedef struct X11_PACKED x11_shm_query_version_request_t {
    uint8_t major_opcode;
    uint8_t minor_opcode;
    uint16_t length;
} x11_shm_query_version_request_t;

typedef struct X11_PACKED x11_shm_query_version_reply_t {
    uint8_t reply;
    uint8_t shared_pixmaps;
    uint16_t sequence_number;
    uint32_t reply_length;
    uint16_t major_version;
    uint16_t minor_version;
    uint16_t uid;
    uint16_t gid;
    uint8_t pixmap_format;
    uint8_t pad0[15];
} x11_shm_query_version_reply_t;

typedef struct X11_PACKED x11_shm_detach_request_t {
    uint8_t major_opcode;
    uint8_t minor_opcode;
    uint16_t length;
    uint32_t shmseg;
} x11_shm_detach_request_t;

typedef struct X11_PACKED x11_shm_put_image_request_t {
    uint8_t major_opcode;
    uint8_t minor_opcode;
    uint16_t length;
    uint32_t drawable;
    uint32_t gc;
    uint16_t total_width;
    uint16_t total_height;
    uint16_t src_x;
    uint16_t src_y;
    uint16_t src_width;
    uint16_t src_height;
    int16_t dst_x;
    int16_t dst_y;
    uint8_t depth;
    uint8_t format;
    uint8_t send_event;
    uint8_t pad0;
    uint32_t shmseg;
    uint32_t offset;
} x11_shm_put_image_request_t;

typedef struct X11_PACKED x11_get_atom_name_request_t {
    uint8_t major_opcode;
    uint8_t pad0;
    uint16_t length;
    uint32_t atom;
} x11_get_atom_name_request_t;

typedef struct X11_PACKED x11_get_atom_name_reply_t {
    uint8_t reply;
    uint8_t pad0;
    uint16_t sequence_number;
    uint32_t reply_length;
    uint16_t name_len;
    uint8_t pad1[22];
} x11_get_atom_name_reply_t;

typedef struct X11_PACKED x11_randr_query_version_request_t {
    uint8_t major_opcode;
    uint8_t minor_opcode;
    uint16_t length;
    uint32_t major_version;
    uint32_t minor_version;
} x11_randr_query_version_request_t;

typedef struct X11_PACKED x11_randr_query_version_reply_t {
    uint8_t reply;
    uint8_t pad0;
    uint16_t sequence_number;
    uint32_t reply_length;
    uint32_t server_major;
    uint32_t server_minor;
    uint8_t pad1[16];
} x11_randr_query_version_reply_t;

typedef struct X11_PACKED x11_randr_get_screen_resources_request_t {
    uint8_t major_opcode;
    uint8_t minor_opcode;
    uint16_t length;
    uint32_t window;
} x11_randr_get_screen_resources_request_t;

typedef struct X11_PACKED x11_randr_get_screen_resources_reply_t {
    uint8_t reply;
    uint8_t pad0;
    uint16_t sequence_number;
    uint32_t reply_length;
    uint32_t timestamp;
    uint32_t config_timestamp;
    uint16_t ncrtcs;
    uint16_t noutputs;
    uint16_t nmodes;
    uint16_t names_len;
    uint8_t pad1[8];
} x11_randr_get_screen_resources_reply_t;

typedef struct X11_PACKED x11_randr_get_crtc_info_request_t {
    uint8_t major_opcode;
    uint8_t minor_opcode;
    uint16_t length;
    uint32_t crtc;
    uint32_t config_timestamp;
} x11_randr_get_crtc_info_request_t;

typedef struct X11_PACKED x11_randr_get_crtc_info_reply_t {
    uint8_t reply;
    uint8_t status;
    uint16_t sequence_number;
    uint32_t reply_length;
    uint32_t timestamp;
    int16_t x;
    int16_t y;
    uint16_t width;
    uint16_t height;
    uint32_t mode;
    uint16_t rotation;
    uint16_t rotations;
    uint16_t noutputs;
    uint16_t npossible;
} x11_randr_get_crtc_info_reply_t;

typedef struct X11_PACKED x11_randr_get_monitors_request_t {
    uint8_t major_opcode;
    uint8_t minor_opcode;
    uint16_t length;
    uint32_t window;
    uint8_t get_active;
    uint8_t pad0[3];
} x11_randr_get_monitors_request_t;

typedef struct X11_PACKED x11_randr_get_monitors_reply_t {
    uint8_t reply;
    uint8_t pad0;
    uint16_t sequence_number;
    uint32_t reply_length;
    uint32_t timestamp;
    uint32_t n_monitors;
    uint32_t n_outputs;
    uint8_t pad1[12];
} x11_randr_get_monitors_reply_t;

typedef struct X11_PACKED x11_randr_monitor_info_t {
    uint32_t name;      // atom
    uint8_t primary;
    uint8_t automatic;
    uint16_t n_output;
    int16_t x;
    int16_t y;
    uint16_t width;     // pixels
    uint16_t height;    // pixels
    uint32_t width_mm;
    uint32_t height_mm;
} x11_randr_monitor_info_t;

// X11 structs
typedef struct x11_connection_t {
    int32_t fd;
    uint32_t id;
    uint32_t id_inc;
    uint32_t max_request_len;  // in 4-byte units; >65535 means BigRequests is active
    x11_screen_t screen;
    uint32_t wm_protocols;
    uint32_t wm_delete_window;
    uint32_t net_wm_name;
    uint32_t net_wm_pid;
    uint32_t net_wm_window_type;
    uint32_t net_wm_window_type_normal;
    uint32_t utf8_string;
    bool has_shm;
    uint8_t shm_opcode;
    bool has_randr;
    uint8_t randr_opcode;
    uint32_t randr_major;
    uint32_t randr_minor;
} x11_connection_t;

typedef struct x11_event_t {
    uint8_t type;
    uint16_t expose_count;
    uint16_t configure_width;
    uint16_t configure_height;
} x11_event_t;

// Image backed by a pixel buffer (MIT-SHM or heap)
typedef struct x11_image_t {
    uint32_t gc;
    uint32_t shmseg;   // 0 if not using SHM
    int32_t shmid;     // -1 if not using SHM
    uint32_t* pixels;  // 0x00RRGGBB row-major pixel buffer
    int32_t width;
    int32_t height;
    int32_t capacity;  // allocated pixel count; reuse when new_w*new_h <= capacity
} x11_image_t;

typedef struct x11_monitor_t {
    int16_t x;
    int16_t y;
    uint16_t width;
    uint16_t height;
    bool primary;
    char name[256];
} x11_monitor_t;

bool x11_connect(x11_connection_t* conn);

uint32_t x11_generate_id(x11_connection_t* conn);

void x11_create_window(x11_connection_t* conn, uint8_t depth, uint32_t wid, uint32_t parent, int16_t x, int16_t y,
                       uint16_t width, uint16_t height, uint16_t border_width, uint16_t _class, uint32_t visual,
                       uint32_t value_mask, uint32_t* value_list, size_t value_list_size);

void x11_change_property(x11_connection_t* conn, uint8_t mode, uint32_t window, uint32_t property, uint32_t type,
                         uint8_t format, void* data, size_t data_size);

void x11_configure_window(x11_connection_t* conn, uint32_t window, uint32_t value_mask, uint32_t* value_list,
                          size_t value_list_size);

void x11_set_wm_protocols(x11_connection_t* conn, uint32_t window);

void x11_set_wm_hints(x11_connection_t* conn, uint32_t window);

void x11_map_window(x11_connection_t* conn, uint32_t window);

// Create a framebuffer image backed by MIT-SHM shared memory (or heap fallback)
bool x11_create_image(x11_connection_t* conn, x11_image_t* img, uint32_t window, int32_t width, int32_t height);

// Resize image to new dimensions, reusing the pixel buffer when capacity allows.
// Keeps the GC alive — much cheaper than destroy + create during window resize.
bool x11_resize_image(x11_connection_t* conn, x11_image_t* img, int32_t new_w, int32_t new_h);

// Blit the full image to the window at (0, 0) using ShmPutImage or PutImage
void x11_put_image(x11_connection_t* conn, uint32_t window, x11_image_t* img);

// Free image resources (SHM or heap)
void x11_destroy_image(x11_connection_t* conn, x11_image_t* img);

bool x11_wait_for_event(x11_connection_t* conn, x11_event_t* event);

// Enumerate monitors via RANDR; returns heap-allocated array of count entries.
// Returns false if RANDR is unavailable. Free with x11_randr_free_monitors().
bool x11_randr_get_monitors(x11_connection_t* conn, x11_monitor_t** monitors, int32_t* count);

void x11_randr_free_monitors(x11_monitor_t* monitors);

void x11_disconnect(x11_connection_t* conn);
