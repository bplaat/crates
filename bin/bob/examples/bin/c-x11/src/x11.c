/*
 * Copyright (c) 2022-2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

#include "x11.h"

#include <arpa/inet.h>
#include <errno.h>
#include <fcntl.h>
#include <netdb.h>
#include <netinet/in.h>
#include <signal.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/socket.h>
#include <sys/un.h>
#include <unistd.h>

// MARK: Helpers

static bool x11_write_all(int fd, const void* buf, size_t len) {
    const uint8_t* p = buf;
    while (len > 0) {
        ssize_t n = write(fd, p, len);
        if (n <= 0) {
            if (n < 0 && errno == EINTR)
                continue;
            return false;
        }
        p += n;
        len -= n;
    }
    return true;
}

static bool x11_read_all(int fd, void* buf, size_t len) {
    uint8_t* p = buf;
    while (len > 0) {
        ssize_t n = read(fd, p, len);
        if (n <= 0) {
            if (n < 0 && errno == EINTR)
                continue;
            return false;
        }
        p += n;
        len -= n;
    }
    return true;
}

static bool x11_write_padded(int fd, const void* data, size_t len) {
    if (!x11_write_all(fd, data, len))
        return false;
    size_t pad = (4 - (len % 4)) % 4;
    if (pad > 0) {
        uint8_t zeros[3] = {0};
        if (!x11_write_all(fd, zeros, pad))
            return false;
    }
    return true;
}

static bool x11_skip(int fd, size_t len) {
    uint8_t buf[64];
    while (len > 0) {
        size_t chunk = len < sizeof(buf) ? len : sizeof(buf);
        if (!x11_read_all(fd, buf, chunk))
            return false;
        len -= chunk;
    }
    return true;
}

static bool x11_intern_atom(int fd, const char* name) {
    size_t name_len = strlen(name);
    size_t padded = (name_len + 3) & ~3;
    x11_intern_atom_request_t req = {
        .major_opcode = X11_INTERN_ATOM,
        .only_if_exists = 0,
        .length = (sizeof(x11_intern_atom_request_t) + padded) / 4,
        .name_len = name_len,
    };
    if (!x11_write_all(fd, &req, sizeof(req)))
        return false;
    return x11_write_padded(fd, name, name_len);
}

static bool x11_read_intern_atom_reply(int fd, uint32_t* atom) {
    x11_intern_atom_reply_t reply;
    if (!x11_read_all(fd, &reply, sizeof(reply)))
        return false;
    if (reply.reply != 1)
        return false;
    *atom = reply.atom;
    return true;
}

// MARK: Xauthority parsing

typedef struct {
    uint16_t name_len;
    char* name;
    uint16_t data_len;
    uint8_t* data;
} x11_cookie_t;

static x11_cookie_t* parse_xauthority(const char* xauth_path, int display_number) {
    FILE* xauth_file = fopen(xauth_path, "rb");
    if (!xauth_file)
        return NULL;

    x11_cookie_t* result = NULL;
    while (1) {
        uint16_t family, address_len, display_len, local_name_len, local_data_len;
        if (fread(&family, 2, 1, xauth_file) != 1)
            break;
        family = ntohs(family);

        if (fread(&address_len, 2, 1, xauth_file) != 1)
            break;
        address_len = ntohs(address_len);
        if (fseek(xauth_file, address_len, SEEK_CUR) != 0)
            break;

        // Read and match display number
        if (fread(&display_len, 2, 1, xauth_file) != 1)
            break;
        display_len = ntohs(display_len);
        char display_str[33];
        int entry_display = -1;
        if (display_len > 32) {
            if (fseek(xauth_file, display_len, SEEK_CUR) != 0)
                break;
        } else {
            if (fread(display_str, 1, display_len, xauth_file) != display_len)
                break;
            display_str[display_len] = '\0';
            entry_display = atoi(display_str);
        }

        if (fread(&local_name_len, 2, 1, xauth_file) != 1)
            break;
        local_name_len = ntohs(local_name_len);
        if (local_name_len > 256)
            break;
        char* local_name = malloc(local_name_len + 1);
        if (!local_name)
            break;
        if (fread(local_name, 1, local_name_len, xauth_file) != local_name_len) {
            free(local_name);
            break;
        }

        if (fread(&local_data_len, 2, 1, xauth_file) != 1) {
            free(local_name);
            break;
        }
        local_data_len = ntohs(local_data_len);
        if (local_data_len > 256) {
            free(local_name);
            break;
        }
        uint8_t* local_data = malloc(local_data_len + 1);
        if (!local_data) {
            free(local_name);
            break;
        }
        if (fread(local_data, 1, local_data_len, xauth_file) != local_data_len) {
            free(local_name);
            free(local_data);
            break;
        }

        if (entry_display == display_number && local_name_len == 18 &&
            memcmp(local_name, "MIT-MAGIC-COOKIE-1", 18) == 0) {
            result = malloc(sizeof(x11_cookie_t));
            if (result) {
                result->name_len = local_name_len;
                result->name = local_name;
                result->data_len = local_data_len;
                result->data = local_data;
            } else {
                free(local_name);
                free(local_data);
            }
            break;
        }
        free(local_name);
        free(local_data);
    }
    fclose(xauth_file);
    return result;
}

static void free_cookie(x11_cookie_t* cookie) {
    if (cookie) {
        free(cookie->name);
        free(cookie->data);
        free(cookie);
    }
}

// MARK: Connection

bool x11_connect(x11_connection_t* conn) {
    // Ignore SIGPIPE so writes to a dead server don't kill us
    signal(SIGPIPE, SIG_IGN);

    // Parse DISPLAY (formats: ":0", "host:0", "/path/to/socket:0")
    char host[256] = "";
    char socket_path[256] = "";
    int display_number = 0;
    const char* display_env = getenv("DISPLAY");
    if (display_env) {
        const char* colon = strrchr(display_env, ':');
        if (colon) {
            size_t host_len = colon - display_env;
            if (host_len > 0 && host_len < sizeof(host)) {
                memcpy(host, display_env, host_len);
                host[host_len] = '\0';
            }
            display_number = atoi(colon + 1);
        }
    }

    // If host contains '/', it's a path-based display (e.g. XQuartz)
    // The actual socket file may include the ":N" suffix in its name
    bool use_tcp = false;
    if (strchr(host, '/')) {
        // First try the full DISPLAY string as the socket path (XQuartz uses "path:0" as filename)
        snprintf(socket_path, sizeof(socket_path), "%s", display_env ? display_env : "");
        host[0] = '\0';
    } else if (host[0] != '\0' && strcmp(host, "localhost") != 0) {
        use_tcp = true;
    }

    // Try to read and parse .Xauthority
    char xauth_path[512];
    const char* xauth_env = getenv("XAUTHORITY");
    if (xauth_env) {
        snprintf(xauth_path, sizeof(xauth_path), "%s", xauth_env);
    } else {
        const char* home = getenv("HOME");
        snprintf(xauth_path, sizeof(xauth_path), "%s/.Xauthority", home ? home : "");
    }
    x11_cookie_t* cookie = parse_xauthority(xauth_path, display_number);

    // Connect to X11 server
    if (use_tcp) {
        char port_str[16];
        snprintf(port_str, sizeof(port_str), "%d", 6000 + display_number);
        struct addrinfo hints = {.ai_family = AF_UNSPEC, .ai_socktype = SOCK_STREAM};
        struct addrinfo* res;
        if (getaddrinfo(host, port_str, &hints, &res) != 0) {
            free_cookie(cookie);
            return false;
        }
        conn->fd = -1;
        for (struct addrinfo* ai = res; ai; ai = ai->ai_next) {
            conn->fd = socket(ai->ai_family, ai->ai_socktype, ai->ai_protocol);
            if (conn->fd < 0)
                continue;
            if (connect(conn->fd, ai->ai_addr, ai->ai_addrlen) == 0)
                break;
            close(conn->fd);
            conn->fd = -1;
        }
        freeaddrinfo(res);
        if (conn->fd < 0) {
            free_cookie(cookie);
            return false;
        }
    } else {
#ifdef SOCK_CLOEXEC
        conn->fd = socket(AF_UNIX, SOCK_STREAM | SOCK_CLOEXEC, 0);
#else
        conn->fd = socket(AF_UNIX, SOCK_STREAM, 0);
#endif
        if (conn->fd < 0) {
            free_cookie(cookie);
            return false;
        }
#ifndef SOCK_CLOEXEC
        fcntl(conn->fd, F_SETFD, FD_CLOEXEC);
#endif

        if (socket_path[0] == '\0') {
            snprintf(socket_path, sizeof(socket_path), "/tmp/.X11-unix/X%d", display_number);
        }

        struct sockaddr_un saddr = {.sun_family = AF_UNIX};
        bool connected = false;

        // Try XDG_RUNTIME_DIR socket path first (some distros/containers use this)
        const char* xdg_runtime = getenv("XDG_RUNTIME_DIR");
        if (xdg_runtime) {
            char xdg_socket[256];
            snprintf(xdg_socket, sizeof(xdg_socket), "%s/.X11-unix/X%d", xdg_runtime, display_number);
            memset(&saddr, 0, sizeof(saddr));
            saddr.sun_family = AF_UNIX;
            snprintf(saddr.sun_path, sizeof(saddr.sun_path), "%s", xdg_socket);
            if (connect(conn->fd, (struct sockaddr*)&saddr, sizeof(saddr)) == 0) {
                connected = true;
            }
        }

#ifdef __linux__
        if (!connected) {
            // Try abstract socket (Linux-specific)
            memset(saddr.sun_path, 0, sizeof(saddr.sun_path));
            size_t path_len = strlen(socket_path);
            if (path_len + 1 <= sizeof(saddr.sun_path)) {
                saddr.sun_path[0] = '\0';
                memcpy(saddr.sun_path + 1, socket_path, path_len);
                socklen_t addr_len = offsetof(struct sockaddr_un, sun_path) + 1 + path_len;
                if (connect(conn->fd, (struct sockaddr*)&saddr, addr_len) == 0) {
                    connected = true;
                }
            }
        }
#endif

        if (!connected) {
            // Try filesystem socket
            memset(&saddr, 0, sizeof(saddr));
            saddr.sun_family = AF_UNIX;
            snprintf(saddr.sun_path, sizeof(saddr.sun_path), "%s", socket_path);
            if (connect(conn->fd, (struct sockaddr*)&saddr, sizeof(saddr)) < 0) {
                close(conn->fd);
                free_cookie(cookie);
                return false;
            }
        }
    }

    // Set CLOEXEC for TCP sockets too
    if (use_tcp) {
        fcntl(conn->fd, F_SETFD, FD_CLOEXEC);
    }

    // Send setup request message
    x11_setup_request_t setup_request = {
#ifdef X11_BIG_ENDIAN
        .byte_order = 'B',
#else
        .byte_order = 'l',
#endif
        .protocol_major_version = 11,
        .protocol_minor_version = 0,
        .authorization_protocol_name_len = cookie ? cookie->name_len : 0,
        .authorization_protocol_data_len = cookie ? cookie->data_len : 0,
    };
    if (!x11_write_all(conn->fd, &setup_request, sizeof(x11_setup_request_t)))
        goto fail;
    if (cookie) {
        if (!x11_write_padded(conn->fd, cookie->name, cookie->name_len))
            goto fail;
        if (!x11_write_padded(conn->fd, cookie->data, cookie->data_len))
            goto fail;
        free_cookie(cookie);
        cookie = NULL;
    }

    // Read setup
    x11_setup_t setup;
    if (!x11_read_all(conn->fd, &setup, sizeof(setup)))
        goto fail;
    if (setup.status != 1)
        goto fail;
    conn->id = setup.resource_id_base;
    conn->id_inc = setup.resource_id_mask & -(setup.resource_id_mask);

    // Read vendor (padded to 4-byte boundary)
    size_t vendor_padded = (setup.vendor_len + 3) & ~3;
    if (!x11_skip(conn->fd, vendor_padded))
        goto fail;

    // Read formats
    if (!x11_skip(conn->fd, setup.pixmap_formats_len * sizeof(x11_format_t)))
        goto fail;

    // Read screens
    x11_screen_t unused_screen;
    for (int32_t i = 0; i < setup.roots_len; i++) {
        x11_screen_t* screen = i == 0 ? &conn->screen : &unused_screen;
        if (!x11_read_all(conn->fd, screen, sizeof(x11_screen_t)))
            goto fail;

        for (int32_t j = 0; j < screen->allowed_depths_len; j++) {
            x11_depth_t unused_depth;
            if (!x11_read_all(conn->fd, &unused_depth, sizeof(x11_depth_t)))
                goto fail;
            if (!x11_skip(conn->fd, unused_depth.visuals_len * sizeof(x11_visualtype_t)))
                goto fail;
        }
    }

    // Intern atoms
    if (!x11_intern_atom(conn->fd, "WM_PROTOCOLS"))
        goto fail;
    if (!x11_intern_atom(conn->fd, "WM_DELETE_WINDOW"))
        goto fail;
    if (!x11_intern_atom(conn->fd, "_NET_WM_NAME"))
        goto fail;
    if (!x11_intern_atom(conn->fd, "_NET_WM_PID"))
        goto fail;
    if (!x11_intern_atom(conn->fd, "UTF8_STRING"))
        goto fail;

    if (!x11_read_intern_atom_reply(conn->fd, &conn->wm_protocols))
        goto fail;
    if (!x11_read_intern_atom_reply(conn->fd, &conn->wm_delete_window))
        goto fail;
    if (!x11_read_intern_atom_reply(conn->fd, &conn->net_wm_name))
        goto fail;
    if (!x11_read_intern_atom_reply(conn->fd, &conn->net_wm_pid))
        goto fail;
    if (!x11_read_intern_atom_reply(conn->fd, &conn->utf8_string))
        goto fail;

    return true;

fail:
    close(conn->fd);
    free_cookie(cookie);
    return false;
}

uint32_t x11_generate_id(x11_connection_t* conn) {
    uint32_t id = conn->id;
    conn->id += conn->id_inc;
    return id;
}

void x11_open_font(x11_connection_t* conn, uint32_t fid, const char* name, size_t name_size) {
    size_t aligned_name_size = (name_size + 3) & ~3;
    x11_open_font_request_t open_font_request = {
        .major_opcode = X11_OPEN_FONT,
        .length = (sizeof(x11_open_font_request_t) + aligned_name_size) / 4,
        .fid = fid,
        .name_len = name_size,
    };
    x11_write_all(conn->fd, &open_font_request, sizeof(x11_open_font_request_t));
    x11_write_padded(conn->fd, name, name_size);
}

void x11_close_font(x11_connection_t* conn, uint32_t font) {
    x11_close_font_request_t close_font_request = {
        .major_opcode = X11_CLOSE_FONT, .length = (sizeof(x11_close_font_request_t)) / 4, .font = font};
    x11_write_all(conn->fd, &close_font_request, sizeof(x11_close_font_request_t));
}

void x11_create_gc(x11_connection_t* conn, uint32_t cid, uint32_t drawable, uint32_t value_mask, uint32_t* value_list,
                   size_t value_list_size) {
    x11_create_gc_request_t create_gc_request = {.major_opcode = X11_CREATE_GC,
                                                 .length = (sizeof(x11_create_gc_request_t) + value_list_size) / 4,
                                                 .cid = cid,
                                                 .drawable = drawable,
                                                 .value_mask = value_mask};
    x11_write_all(conn->fd, &create_gc_request, sizeof(x11_create_gc_request_t));
    x11_write_all(conn->fd, value_list, value_list_size);
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
    x11_write_all(conn->fd, &create_window_request, sizeof(x11_create_window_request_t));
    x11_write_all(conn->fd, value_list, value_list_size);
}

void x11_change_property(x11_connection_t* conn, uint8_t mode, uint32_t window, uint32_t property, uint32_t type,
                         uint8_t format, void* data, size_t data_size) {
    size_t aligned_data_size = (data_size + 3) & ~3;
    uint32_t data_len_in_format_units = data_size;
    if (format == 16)
        data_len_in_format_units = data_size / 2;
    else if (format == 32)
        data_len_in_format_units = data_size / 4;
    x11_change_property_request_t change_property_request = {
        .major_opcode = X11_CHANGE_PROPERTY,
        .mode = mode,
        .length = (sizeof(x11_change_property_request_t) + aligned_data_size) / 4,
        .window = window,
        .property = property,
        .type = type,
        .format = format,
        .data_len = data_len_in_format_units};
    x11_write_all(conn->fd, &change_property_request, sizeof(x11_change_property_request_t));
    x11_write_padded(conn->fd, data, data_size);
}

void x11_configure_window(x11_connection_t* conn, uint32_t window, uint32_t value_mask, uint32_t* value_list,
                          size_t value_list_size) {
    x11_configure_window_request_t configure_window_request = {
        .major_opcode = X11_CONFIGURE_WINDOW,
        .length = (sizeof(x11_configure_window_request_t) + value_list_size) / 4,
        .window = window,
        .value_mask = value_mask};
    x11_write_all(conn->fd, &configure_window_request, sizeof(x11_configure_window_request_t));
    x11_write_all(conn->fd, value_list, value_list_size);
}

void x11_set_wm_protocols(x11_connection_t* conn, uint32_t window) {
    uint32_t atoms[] = {conn->wm_delete_window};
    x11_change_property(conn, X11_PROP_MODE_REPLACE, window, conn->wm_protocols, X11_ATOM_ATOM, 32, atoms,
                        sizeof(atoms));
}

void x11_set_wm_hints(x11_connection_t* conn, uint32_t window) {
    // WM_HINTS: flags=InputHint|StateHint, input=True, initial_state=NormalState
    uint32_t wm_hints[9] = {0};
    wm_hints[0] = 1 | 2;  // flags: InputHint | StateHint
    wm_hints[1] = 1;       // input: True
    wm_hints[2] = 1;       // initial_state: NormalState
    x11_change_property(conn, X11_PROP_MODE_REPLACE, window, X11_ATOM_WM_HINTS, X11_ATOM_WM_HINTS, 32, wm_hints,
                        sizeof(wm_hints));
}

void x11_map_window(x11_connection_t* conn, uint32_t window) {
    x11_map_window_request_t map_window_request = {
        .major_opcode = X11_MAP_WINDOW, .length = sizeof(x11_map_window_request_t) / 4, .window = window};
    x11_write_all(conn->fd, &map_window_request, sizeof(x11_map_window_request_t));
}

void x11_poly_rectangle(x11_connection_t* conn, uint32_t drawable, uint32_t gc, x11_rectangle_t* rectangles,
                        size_t rectangles_size) {
    x11_poly_rectangle_request_t poly_rectangle_request = {
        .major_opcode = X11_POLY_RECTANGLE,
        .length = (sizeof(x11_poly_rectangle_request_t) + rectangles_size) / 4,
        .drawable = drawable,
        .gc = gc};
    x11_write_all(conn->fd, &poly_rectangle_request, sizeof(x11_poly_rectangle_request_t));
    x11_write_all(conn->fd, rectangles, rectangles_size);
}

void x11_image_text_8(x11_connection_t* conn, uint32_t drawable, uint32_t gc, int16_t x, int16_t y, const char* string,
                      size_t string_size) {
    if (string_size > 255)
        string_size = 255;
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
    x11_write_all(conn->fd, &image_text_8_request, sizeof(x11_image_text_8_request_t));
    x11_write_padded(conn->fd, string, string_size);
}

bool x11_wait_for_event(x11_connection_t* conn, x11_event_t* event) {
    uint8_t buf[32];
    if (!x11_read_all(conn->fd, buf, 32))
        return false;
    event->type = buf[0] & ~0x80;
    event->expose_count = 0;

    if (event->type == X11_ERROR) {
        uint8_t error_code = buf[1];
        uint16_t seq;
        memcpy(&seq, &buf[2], 2);
        fprintf(stderr, "X11 error: code=%d sequence=%d\n", error_code, seq);
        return true;
    }

    if (event->type == X11_EXPOSE) {
        memcpy(&event->expose_count, &buf[16], 2);
    }

    if (event->type == X11_CLIENT_MESSAGE) {
        uint32_t msg_type;
        memcpy(&msg_type, &buf[8], 4);
        if (msg_type == conn->wm_protocols) {
            uint32_t data0;
            memcpy(&data0, &buf[12], 4);
            if (data0 == conn->wm_delete_window) {
                event->type = X11_CLIENT_MESSAGE_CLOSE;
            }
        }
    }
    return true;
}

void x11_disconnect(x11_connection_t* conn) {
    close(conn->fd);
}
