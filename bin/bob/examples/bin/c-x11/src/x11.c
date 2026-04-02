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
#include <netinet/tcp.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/ipc.h>
#include <sys/shm.h>
#include <sys/socket.h>
#include <sys/uio.h>
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

static bool x11_query_extension(int fd, const char* name) {
    size_t name_len = strlen(name);
    size_t padded = (name_len + 3) & ~3;
    x11_query_extension_request_t req = {
        .major_opcode = X11_QUERY_EXTENSION,
        .length = (uint16_t)((sizeof(x11_query_extension_request_t) + padded) / 4),
        .name_len = (uint16_t)name_len,
    };
    if (!x11_write_all(fd, &req, sizeof(req)))
        return false;
    return x11_write_padded(fd, name, name_len);
}

static bool x11_read_query_extension_reply(int fd, bool* present, uint8_t* major_opcode, uint8_t* first_event) {
    x11_query_extension_reply_t reply;
    if (!x11_read_all(fd, &reply, sizeof(reply)))
        return false;
    if (reply.reply != 1)
        return false;
    *present = reply.present != 0;
    *major_opcode = reply.major_opcode;
    if (first_event)
        *first_event = reply.first_event;
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
        if (local_name_len > 256) {
            if (fseek(xauth_file, local_name_len, SEEK_CUR) != 0)
                break;
            // Skip data length + data
            uint16_t skip_data_len;
            if (fread(&skip_data_len, 2, 1, xauth_file) != 1)
                break;
            skip_data_len = ntohs(skip_data_len);
            if (fseek(xauth_file, skip_data_len, SEEK_CUR) != 0)
                break;
            continue;
        }
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
            if (fseek(xauth_file, local_data_len, SEEK_CUR) != 0)
                break;
            continue;
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

static float parse_xft_dpi(const char* resources, uint32_t len) {
    const char key[] = "Xft.dpi:";
    uint32_t key_len = 8;
    for (uint32_t i = 0; i + key_len <= len; i++) {
        if (memcmp(resources + i, key, key_len) != 0)
            continue;
        const char* p = resources + i + key_len;
        while (*p == ' ' || *p == '\t') p++;
        double val = atof(p);
        if (val > 0.0)
            return (float)val;
    }
    return 0.0f;
}

// MARK: Connection

bool x11_connect(x11_connection_t* conn) {
    memset(conn, 0, sizeof(*conn));
    conn->max_request_len = 65535;  // default before BigRequests
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
#ifdef SOCK_CLOEXEC
            conn->fd = socket(ai->ai_family, ai->ai_socktype | SOCK_CLOEXEC, ai->ai_protocol);
#else
            conn->fd = socket(ai->ai_family, ai->ai_socktype, ai->ai_protocol);
            if (conn->fd >= 0)
                fcntl(conn->fd, F_SETFD, FD_CLOEXEC);
#endif
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
        int flag = 1;
        setsockopt(conn->fd, IPPROTO_TCP, TCP_NODELAY, &flag, sizeof(flag));
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

#ifdef __linux__
        // Try abstract socket first (primary X11 socket on Linux)
        {
            memset(&saddr, 0, sizeof(saddr));
            saddr.sun_family = AF_UNIX;
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

        // Try XDG_RUNTIME_DIR socket path (some distros/containers use this)
        if (!connected) {
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
        }

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

    // No separate CLOEXEC needed — all socket paths now set it at creation time

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
    conn->max_request_len = setup.maximum_request_length;

    // Read vendor (padded to 4-byte boundary)
    size_t vendor_padded = (setup.vendor_len + 3) & ~3;
    if (!x11_skip(conn->fd, vendor_padded))
        goto fail;

    // Read formats
    if (!x11_skip(conn->fd, setup.pixmap_formats_len * sizeof(x11_format_t)))
        goto fail;

    // Read screens; capture root visual masks for pixel-format validation
    bool root_visual_found = false;
    x11_screen_t unused_screen;
    for (int32_t i = 0; i < setup.roots_len; i++) {
        x11_screen_t* screen = i == 0 ? &conn->screen : &unused_screen;
        if (!x11_read_all(conn->fd, screen, sizeof(x11_screen_t)))
            goto fail;

        for (int32_t j = 0; j < screen->allowed_depths_len; j++) {
            x11_depth_t depth;
            if (!x11_read_all(conn->fd, &depth, sizeof(x11_depth_t)))
                goto fail;
            for (int32_t k = 0; k < depth.visuals_len; k++) {
                x11_visualtype_t vis;
                if (!x11_read_all(conn->fd, &vis, sizeof(x11_visualtype_t)))
                    goto fail;
                if (i == 0 && !root_visual_found && vis.visual_id == conn->screen.root_visual) {
                    root_visual_found = true;
                    conn->root_visual_red_mask = vis.red_mask;
                    conn->root_visual_green_mask = vis.green_mask;
                    conn->root_visual_blue_mask = vis.blue_mask;
                }
            }
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
    if (!x11_intern_atom(conn->fd, "_NET_WM_WINDOW_TYPE"))
        goto fail;
    if (!x11_intern_atom(conn->fd, "_NET_WM_WINDOW_TYPE_NORMAL"))
        goto fail;
    if (!x11_intern_atom(conn->fd, "_NET_WM_SYNC_REQUEST"))
        goto fail;
    if (!x11_intern_atom(conn->fd, "_NET_WM_SYNC_REQUEST_COUNTER"))
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
    if (!x11_read_intern_atom_reply(conn->fd, &conn->net_wm_window_type))
        goto fail;
    if (!x11_read_intern_atom_reply(conn->fd, &conn->net_wm_window_type_normal))
        goto fail;
    if (!x11_read_intern_atom_reply(conn->fd, &conn->net_wm_sync_request))
        goto fail;
    if (!x11_read_intern_atom_reply(conn->fd, &conn->net_wm_sync_request_counter))
        goto fail;

    // Query MIT-SHM, BIG-REQUESTS, RANDR, and SYNC extensions together (pipelined)
    if (!x11_query_extension(conn->fd, "MIT-SHM"))
        goto fail;
    if (!x11_query_extension(conn->fd, "BIG-REQUESTS"))
        goto fail;
    if (!x11_query_extension(conn->fd, "RANDR"))
        goto fail;
    if (!x11_query_extension(conn->fd, "SYNC"))
        goto fail;
    {
        bool shm_present = false;
        uint8_t shm_opcode = 0;
        if (!x11_read_query_extension_reply(conn->fd, &shm_present, &shm_opcode, NULL))
            goto fail;
        conn->has_shm = shm_present;
        conn->shm_opcode = shm_opcode;
    }
    {
        bool big_present = false;
        uint8_t big_opcode = 0;
        if (!x11_read_query_extension_reply(conn->fd, &big_present, &big_opcode, NULL))
            goto fail;
        if (big_present) {
            x11_big_req_enable_request_t br_req = {
                .major_opcode = big_opcode,
                .minor_opcode = 0,
                .length = sizeof(br_req) / 4,
            };
            if (!x11_write_all(conn->fd, &br_req, sizeof(br_req)))
                goto fail;
            x11_big_req_enable_reply_t br_reply;
            if (!x11_read_all(conn->fd, &br_reply, sizeof(br_reply)) || br_reply.reply != 1)
                goto fail;
            conn->max_request_len = br_reply.maximum_request_length;
        }
    }
    {
        bool randr_present = false;
        uint8_t randr_opcode = 0;
        uint8_t randr_first_event = 0;
        if (!x11_read_query_extension_reply(conn->fd, &randr_present, &randr_opcode, &randr_first_event))
            goto fail;
        conn->has_randr = randr_present;
        conn->randr_opcode = randr_opcode;
        conn->randr_first_event = randr_first_event;
    }
    {
        bool sync_present = false;
        uint8_t sync_opcode = 0;
        if (!x11_read_query_extension_reply(conn->fd, &sync_present, &sync_opcode, NULL))
            goto fail;
        conn->has_sync = sync_present;
        conn->sync_opcode = sync_opcode;
    }

    // Verify SHM is actually functional via a round-trip QueryVersion
    if (conn->has_shm) {
        x11_shm_query_version_request_t qv = {
            .major_opcode = conn->shm_opcode,
            .minor_opcode = 0,
            .length = sizeof(qv) / 4,
        };
        if (!x11_write_all(conn->fd, &qv, sizeof(qv)))
            goto fail;
        x11_shm_query_version_reply_t qv_reply;
        if (!x11_read_all(conn->fd, &qv_reply, sizeof(qv_reply)) || qv_reply.reply != 1) {
            conn->has_shm = false;
        }
    }

    // Query RANDR version
    if (conn->has_randr) {
        x11_randr_query_version_request_t vreq = {
            .major_opcode = conn->randr_opcode,
            .minor_opcode = X11_RANDR_QUERY_VERSION,
            .length = sizeof(vreq) / 4,
            .major_version = 1,
            .minor_version = 5,
        };
        if (!x11_write_all(conn->fd, &vreq, sizeof(vreq)))
            goto fail;
        x11_randr_query_version_reply_t vreply;
        if (!x11_read_all(conn->fd, &vreply, sizeof(vreply)) || vreply.reply != 1) {
            conn->has_randr = false;
        } else {
            conn->randr_major = vreply.server_major;
            conn->randr_minor = vreply.server_minor;
        }
    }

    // Initialize SYNC extension (must be done before using any SYNC requests)
    if (conn->has_sync) {
        x11_sync_initialize_request_t sreq = {
            .major_opcode = conn->sync_opcode,
            .minor_opcode = X11_SYNC_INITIALIZE,
            .length = sizeof(sreq) / 4,
            .major_version = 3,
            .minor_version = 1,
        };
        if (!x11_write_all(conn->fd, &sreq, sizeof(sreq)))
            goto fail;
        x11_sync_initialize_reply_t sreply;
        if (!x11_read_all(conn->fd, &sreply, sizeof(sreply)) || sreply.reply != 1) {
            conn->has_sync = false;
        }
    }

    // Query Xft.dpi from root RESOURCE_MANAGER property
    {
        x11_get_property_request_t prop_req = {
            .major_opcode = X11_GET_PROPERTY,
            ._delete = 0,
            .length = sizeof(prop_req) / 4,
            .window = conn->screen.root,
            .property = X11_ATOM_RESOURCE_MANAGER,
            .type = 0,
            .long_offset = 0,
            .long_length = 1024,
        };
        if (!x11_write_all(conn->fd, &prop_req, sizeof(prop_req)))
            goto fail;
        x11_get_property_reply_t prop_reply;
        if (!x11_read_all(conn->fd, &prop_reply, sizeof(prop_reply)))
            goto fail;
        if (prop_reply.reply == 1 && prop_reply.reply_length > 0) {
            size_t data_bytes = (size_t)prop_reply.reply_length * 4;
            char* buf = malloc(data_bytes + 1);
            if (!buf)
                goto fail;
            if (!x11_read_all(conn->fd, buf, data_bytes)) {
                free(buf);
                goto fail;
            }
            uint32_t value_len = prop_reply.value_len < (uint32_t)data_bytes
                                     ? prop_reply.value_len
                                     : (uint32_t)data_bytes;
            buf[value_len] = '\0';
            conn->xft_dpi = parse_xft_dpi(buf, value_len);
            free(buf);
        }
    }

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
    if (conn->has_sync) {
        uint32_t atoms[] = {conn->wm_delete_window, conn->net_wm_sync_request};
        x11_change_property(conn, X11_PROP_MODE_REPLACE, window, conn->wm_protocols, X11_ATOM_ATOM, 32, atoms,
                            sizeof(atoms));
    } else {
        uint32_t atoms[] = {conn->wm_delete_window};
        x11_change_property(conn, X11_PROP_MODE_REPLACE, window, conn->wm_protocols, X11_ATOM_ATOM, 32, atoms,
                            sizeof(atoms));
    }
}

void x11_set_wm_hints(x11_connection_t* conn, uint32_t window) {
    // WM_HINTS: flags=InputHint|StateHint, input=True, initial_state=NormalState
    uint32_t wm_hints[9] = {0};
    wm_hints[0] = 1 | 2;  // flags: InputHint | StateHint
    wm_hints[1] = 1;      // input: True
    wm_hints[2] = 1;      // initial_state: NormalState
    x11_change_property(conn, X11_PROP_MODE_REPLACE, window, X11_ATOM_WM_HINTS, X11_ATOM_WM_HINTS, 32, wm_hints,
                        sizeof(wm_hints));
}

void x11_map_window(x11_connection_t* conn, uint32_t window) {
    x11_map_window_request_t map_window_request = {
        .major_opcode = X11_MAP_WINDOW, .length = sizeof(x11_map_window_request_t) / 4, .window = window};
    x11_write_all(conn->fd, &map_window_request, sizeof(x11_map_window_request_t));
}

bool x11_create_image(x11_connection_t* conn, x11_image_t* img, uint32_t window, int32_t width, int32_t height) {
    size_t size = (size_t)width * (size_t)height * sizeof(uint32_t);

    // Create a minimal GC (no graphics exposures)
    img->gc = x11_generate_id(conn);
    uint32_t gc_values[] = {0};  // GraphicsExposures = False
    x11_create_gc_request_t gc_req = {
        .major_opcode = X11_CREATE_GC,
        .length = (sizeof(x11_create_gc_request_t) + sizeof(gc_values)) / 4,
        .cid = img->gc,
        .drawable = window,
        .value_mask = X11_GC_GRAPHICS_EXPOSURES,
    };
    x11_write_all(conn->fd, &gc_req, sizeof(gc_req));
    x11_write_all(conn->fd, gc_values, sizeof(gc_values));

    if (conn->has_shm) {
        img->shmid = shmget(IPC_PRIVATE, size, IPC_CREAT | 0600);
        if (img->shmid >= 0) {
            img->pixels = shmat(img->shmid, NULL, 0);
            if (img->pixels == (uint32_t*)-1) {
                shmctl(img->shmid, IPC_RMID, NULL);
                img->shmid = -1;
                img->pixels = NULL;
            }
        }
        if (img->shmid >= 0) {
            img->shmseg = x11_generate_id(conn);
            x11_shm_attach_request_t attach = {
                .major_opcode = conn->shm_opcode,
                .minor_opcode = X11_SHM_ATTACH,
                .length = sizeof(attach) / 4,
                .shmseg = img->shmseg,
                .shmid = (uint32_t)img->shmid,
                .read_only = 0,
            };
            x11_write_all(conn->fd, &attach, sizeof(attach));
            // NOTE: do NOT call shmctl(IPC_RMID) here; the X server needs the shmid to attach.
            // IPC_RMID is called in x11_destroy_image after ShmDetach + shmdt.
        } else {
            conn->has_shm = false;
        }
    }

    if (!conn->has_shm) {
        img->shmid = -1;
        img->shmseg = 0;
        img->pixels = calloc(size, 1);
        if (!img->pixels)
            return false;
    }

    img->width = width;
    img->height = height;
    img->capacity = width * height;
    return true;
}

bool x11_resize_image(x11_connection_t* conn, x11_image_t* img, int32_t new_w, int32_t new_h) {
    int32_t new_cap = new_w * new_h;

    if (new_cap <= img->capacity) {
        // Buffer is large enough — reuse it, just update the visible dimensions
        img->width = new_w;
        img->height = new_h;
        return true;
    }

    // Buffer too small: free old pixels (GC stays alive)
    if (img->shmseg != 0) {
        x11_shm_detach_request_t detach = {
            .major_opcode = conn->shm_opcode,
            .minor_opcode = X11_SHM_DETACH,
            .length = sizeof(detach) / 4,
            .shmseg = img->shmseg,
        };
        x11_write_all(conn->fd, &detach, sizeof(detach));
        if (img->pixels && img->pixels != (uint32_t*)-1)
            shmdt(img->pixels);
        shmctl(img->shmid, IPC_RMID, NULL);
        img->shmseg = 0;
        img->shmid = -1;
        img->pixels = NULL;
    } else {
        free(img->pixels);
        img->pixels = NULL;
    }

    // Allocate with 25% headroom to avoid repeated reallocs during drag-resize
    int32_t alloc_cap = new_cap + (new_cap >> 2);
    size_t size = (size_t)alloc_cap * sizeof(uint32_t);

    if (conn->has_shm) {
        img->shmid = shmget(IPC_PRIVATE, size, IPC_CREAT | 0600);
        if (img->shmid >= 0) {
            img->pixels = shmat(img->shmid, NULL, 0);
            if (img->pixels == (uint32_t*)-1) {
                shmctl(img->shmid, IPC_RMID, NULL);
                img->shmid = -1;
                img->pixels = NULL;
            }
        }
        if (img->shmid >= 0) {
            img->shmseg = x11_generate_id(conn);
            x11_shm_attach_request_t attach = {
                .major_opcode = conn->shm_opcode,
                .minor_opcode = X11_SHM_ATTACH,
                .length = sizeof(attach) / 4,
                .shmseg = img->shmseg,
                .shmid = (uint32_t)img->shmid,
                .read_only = 0,
            };
            x11_write_all(conn->fd, &attach, sizeof(attach));
        } else {
            conn->has_shm = false;
        }
    }

    if (!conn->has_shm) {
        img->shmid = -1;
        img->shmseg = 0;
        img->pixels = calloc(size, 1);
        if (!img->pixels)
            return false;
    }

    img->width = new_w;
    img->height = new_h;
    img->capacity = alloc_cap;
    return true;
}

void x11_put_image(x11_connection_t* conn, uint32_t window, x11_image_t* img) {
    uint8_t depth = conn->screen.root_depth;
    if (conn->has_shm && img->shmseg != 0) {
        x11_shm_put_image_request_t req = {
            .major_opcode = conn->shm_opcode,
            .minor_opcode = X11_SHM_PUT_IMAGE,
            .length = sizeof(req) / 4,
            .drawable = window,
            .gc = img->gc,
            .total_width = (uint16_t)img->width,
            .total_height = (uint16_t)img->height,
            .src_x = 0,
            .src_y = 0,
            .src_width = (uint16_t)img->width,
            .src_height = (uint16_t)img->height,
            .dst_x = 0,
            .dst_y = 0,
            .depth = depth,
            .format = X11_IMAGE_FORMAT_Z_PIXMAP,
            .send_event = 0,
            .shmseg = img->shmseg,
            .offset = 0,
        };
        x11_write_all(conn->fd, &req, sizeof(req));
    } else {
        // PutImage fallback: batch as many rows as fit within max_request_len.
        // With BigRequests (max_request_len > 65535) the entire frame usually fits
        // in a single atomic request, eliminating visible tearing on XQuartz/remote X.
        // writev() combines the header and pixel data into one syscall per chunk.
        uint32_t row_bytes = (uint32_t)img->width * sizeof(uint32_t);
        bool use_big_req = conn->max_request_len > 65535;
        uint32_t header_words = use_big_req ? 7 : 6;  // 28 or 24 bytes
        uint32_t max_rows = (conn->max_request_len - header_words) * 4 / row_bytes;
        if (max_rows < 1)
            max_rows = 1;

        int32_t y = 0;
        while (y < img->height) {
            uint32_t rows = (uint32_t)(img->height - y);
            if (rows > max_rows)
                rows = max_rows;
            uint32_t data_bytes = row_bytes * rows;
            uint32_t req_units = header_words + data_bytes / 4;

            struct iovec iov[2];
            uint8_t header[28];  // large enough for both request variants
            size_t header_size;
            if (use_big_req) {
                x11_put_image_big_request_t req = {
                    .major_opcode = X11_PUT_IMAGE,
                    .format = X11_IMAGE_FORMAT_Z_PIXMAP,
                    .length = 0,
                    .big_length = req_units,
                    .drawable = window,
                    .gc = img->gc,
                    .width = (uint16_t)img->width,
                    .height = (uint16_t)rows,
                    .dst_x = 0,
                    .dst_y = (int16_t)y,
                    .left_pad = 0,
                    .depth = depth,
                };
                memcpy(header, &req, sizeof(req));
                header_size = sizeof(req);
            } else {
                x11_put_image_request_t req = {
                    .major_opcode = X11_PUT_IMAGE,
                    .format = X11_IMAGE_FORMAT_Z_PIXMAP,
                    .length = (uint16_t)req_units,
                    .drawable = window,
                    .gc = img->gc,
                    .width = (uint16_t)img->width,
                    .height = (uint16_t)rows,
                    .dst_x = 0,
                    .dst_y = (int16_t)y,
                    .left_pad = 0,
                    .depth = depth,
                };
                memcpy(header, &req, sizeof(req));
                header_size = sizeof(req);
            }
            iov[0].iov_base = header;
            iov[0].iov_len = header_size;
            iov[1].iov_base = img->pixels + y * img->width;
            iov[1].iov_len = data_bytes;

            size_t total = iov[0].iov_len + iov[1].iov_len;
            size_t sent = 0;
            int iov_idx = 0;
            size_t iov_off = 0;
            while (sent < total) {
                struct iovec viov[2];
                int vcnt = 0;
                size_t remaining = iov[iov_idx].iov_len - iov_off;
                viov[vcnt].iov_base = (uint8_t*)iov[iov_idx].iov_base + iov_off;
                viov[vcnt].iov_len = remaining;
                vcnt++;
                if (iov_idx == 0 && iov[1].iov_len > 0) {
                    viov[vcnt] = iov[1];
                    vcnt++;
                }
                ssize_t n = writev(conn->fd, viov, vcnt);
                if (n <= 0) {
                    if (n < 0 && errno == EINTR)
                        continue;
                    break;
                }
                sent += (size_t)n;
                size_t advance = (size_t)n;
                while (advance > 0 && iov_idx < 2) {
                    size_t avail = iov[iov_idx].iov_len - iov_off;
                    if (advance >= avail) {
                        advance -= avail;
                        iov_idx++;
                        iov_off = 0;
                    } else {
                        iov_off += advance;
                        advance = 0;
                    }
                }
            }
            y += (int32_t)rows;
        }
    }
}

void x11_destroy_image(x11_connection_t* conn, x11_image_t* img) {
    // Free the server-side GC
    x11_free_gc_request_t free_gc = {
        .major_opcode = X11_FREE_GC,
        .length = sizeof(free_gc) / 4,
        .gc = img->gc,
    };
    x11_write_all(conn->fd, &free_gc, sizeof(free_gc));

    if (img->shmseg != 0) {
        x11_shm_detach_request_t detach = {
            .major_opcode = conn->shm_opcode,
            .minor_opcode = X11_SHM_DETACH,
            .length = sizeof(detach) / 4,
            .shmseg = img->shmseg,
        };
        x11_write_all(conn->fd, &detach, sizeof(detach));
        if (img->pixels && img->pixels != (uint32_t*)-1)
            shmdt(img->pixels);
        shmctl(img->shmid, IPC_RMID, NULL);
        img->shmseg = 0;
        img->shmid = -1;
        img->pixels = NULL;
    } else {
        free(img->pixels);
        img->pixels = NULL;
    }
}

bool x11_randr_get_monitors(x11_connection_t* conn, x11_monitor_t** monitors, int32_t* count) {
    if (!conn->has_randr)
        return false;

    // RANDR >= 1.5: use RRGetMonitors (logical monitor objects with names)
    if (conn->randr_minor >= 5) {
        x11_randr_get_monitors_request_t req = {
            .major_opcode = conn->randr_opcode,
            .minor_opcode = X11_RANDR_GET_MONITORS,
            .length = sizeof(req) / 4,
            .window = conn->screen.root,
            .get_active = 1,
        };
        if (!x11_write_all(conn->fd, &req, sizeof(req)))
            return false;

        x11_randr_get_monitors_reply_t reply;
        if (!x11_read_all(conn->fd, &reply, sizeof(reply)) || reply.reply != 1)
            return false;

        uint32_t n = reply.n_monitors;
        if (n == 0) {
            *monitors = NULL;
            *count = 0;
            return true;
        }

        uint32_t* atoms = malloc(n * sizeof(uint32_t));
        x11_monitor_t* result = malloc(n * sizeof(x11_monitor_t));
        if (!atoms || !result) {
            free(atoms);
            free(result);
            return false;
        }

        for (uint32_t i = 0; i < n; i++) {
            x11_randr_monitor_info_t info;
            if (!x11_read_all(conn->fd, &info, sizeof(info))) {
                free(atoms);
                free(result);
                return false;
            }
            atoms[i] = info.name;
            result[i].x = info.x;
            result[i].y = info.y;
            result[i].width = info.width;
            result[i].height = info.height;
            result[i].width_mm = info.width_mm;
            result[i].height_mm = info.height_mm;
            result[i].dpi = info.width_mm > 0 ? (float)info.width * 25.4f / (float)info.width_mm : 96.0f;
            result[i].scale = result[i].dpi / 96.0f;
            result[i].primary = info.primary != 0;
            result[i].name[0] = '\0';
            if (!x11_skip(conn->fd, info.n_output * 4)) {
                free(atoms);
                free(result);
                return false;
            }
        }

        for (uint32_t i = 0; i < n; i++) {
            x11_get_atom_name_request_t name_req = {
                .major_opcode = X11_GET_ATOM_NAME,
                .length = sizeof(name_req) / 4,
                .atom = atoms[i],
            };
            if (!x11_write_all(conn->fd, &name_req, sizeof(name_req))) {
                free(atoms);
                free(result);
                return false;
            }
        }
        for (uint32_t i = 0; i < n; i++) {
            x11_get_atom_name_reply_t name_reply;
            if (!x11_read_all(conn->fd, &name_reply, sizeof(name_reply)) || name_reply.reply != 1) {
                free(atoms);
                free(result);
                return false;
            }
            uint16_t name_len = name_reply.name_len;
            size_t padded = (name_len + 3) & ~3;
            if (name_len > 0) {
                char name_buf[256];
                size_t read_len = padded < sizeof(name_buf) ? padded : sizeof(name_buf) - 1;
                if (!x11_read_all(conn->fd, name_buf, read_len)) {
                    free(atoms);
                    free(result);
                    return false;
                }
                if (padded > read_len)
                    x11_skip(conn->fd, padded - read_len);
                size_t copy_len = name_len < sizeof(result[i].name) - 1 ? name_len : sizeof(result[i].name) - 1;
                memcpy(result[i].name, name_buf, copy_len);
                result[i].name[copy_len] = '\0';
            }
        }

        free(atoms);
        *monitors = result;
        *count = (int32_t)n;
        return true;
    }

    // RANDR 1.2 fallback: enumerate active CRTCs via RRGetScreenResources
    {
        x11_randr_get_screen_resources_request_t sreq = {
            .major_opcode = conn->randr_opcode,
            .minor_opcode = X11_RANDR_GET_SCREEN_RESOURCES,
            .length = sizeof(sreq) / 4,
            .window = conn->screen.root,
        };
        if (!x11_write_all(conn->fd, &sreq, sizeof(sreq)))
            return false;

        x11_randr_get_screen_resources_reply_t sreply;
        if (!x11_read_all(conn->fd, &sreply, sizeof(sreply)) || sreply.reply != 1)
            return false;

        uint16_t ncrtcs = sreply.ncrtcs;
        uint16_t noutputs = sreply.noutputs;
        uint16_t nmodes = sreply.nmodes;
        uint16_t names_len = sreply.names_len;
        uint32_t config_ts = sreply.config_timestamp;

        // Must always consume all variable-length data from the reply
        size_t names_padded = (names_len + 3) & ~3;

        if (ncrtcs == 0) {
            if (!x11_skip(conn->fd, noutputs * 4 + nmodes * 32 + names_padded))
                return false;
            *monitors = NULL;
            *count = 0;
            return true;
        }

        uint32_t* crtcs = malloc(ncrtcs * sizeof(uint32_t));
        if (!crtcs)
            return false;
        if (!x11_read_all(conn->fd, crtcs, ncrtcs * sizeof(uint32_t))) {
            free(crtcs);
            return false;
        }
        // Skip outputs, modes, names
        if (!x11_skip(conn->fd, noutputs * 4 + nmodes * 32 + names_padded)) {
            free(crtcs);
            return false;
        }

        // Pipeline all RRGetCrtcInfo requests
        for (uint16_t i = 0; i < ncrtcs; i++) {
            x11_randr_get_crtc_info_request_t creq = {
                .major_opcode = conn->randr_opcode,
                .minor_opcode = X11_RANDR_GET_CRTC_INFO,
                .length = sizeof(creq) / 4,
                .crtc = crtcs[i],
                .config_timestamp = config_ts,
            };
            if (!x11_write_all(conn->fd, &creq, sizeof(creq))) {
                free(crtcs);
                return false;
            }
        }
        free(crtcs);

        // Read all RRGetCrtcInfo replies, collecting active CRTCs
        x11_monitor_t* result = malloc(ncrtcs * sizeof(x11_monitor_t));
        if (!result)
            return false;

        int32_t active = 0;
        for (uint16_t i = 0; i < ncrtcs; i++) {
            x11_randr_get_crtc_info_reply_t creply;
            if (!x11_read_all(conn->fd, &creply, sizeof(creply)) || creply.reply != 1) {
                free(result);
                return false;
            }
            // Skip variable-length output lists
            if (!x11_skip(conn->fd, (creply.noutputs + creply.npossible) * 4)) {
                free(result);
                return false;
            }
            // CRTC is active when it has a mode and nonzero size
            if (creply.mode == 0 || creply.width == 0 || creply.height == 0)
                continue;

            result[active].x = creply.x;
            result[active].y = creply.y;
            result[active].width = creply.width;
            result[active].height = creply.height;
            result[active].width_mm = 0;
            result[active].height_mm = 0;
            // Approximate per-CRTC DPI by scaling the screen physical width proportionally
            float screen_dpi = conn->screen.width_in_millimeters > 0 ? (float)conn->screen.width_in_pixels * 25.4f /
                                                                           (float)conn->screen.width_in_millimeters
                                                                     : 96.0f;
            result[active].dpi = screen_dpi;
            result[active].scale = screen_dpi / 96.0f;
            result[active].primary = false;
            snprintf(result[active].name, sizeof(result[active].name), "CRTC-%d", i);
            active++;
        }

        // Mark the CRTC at (0,0) as primary; fall back to first active
        bool found_primary = false;
        for (int32_t i = 0; i < active; i++) {
            if (result[i].x == 0 && result[i].y == 0) {
                result[i].primary = true;
                found_primary = true;
                break;
            }
        }
        if (!found_primary && active > 0)
            result[0].primary = true;

        *monitors = result;
        *count = active;
        return true;
    }
}

void x11_randr_free_monitors(x11_monitor_t* monitors) {
    free(monitors);
}

void x11_randr_select_input(x11_connection_t* conn, uint32_t window) {
    if (!conn->has_randr)
        return;
    x11_randr_select_input_request_t req = {
        .major_opcode = conn->randr_opcode,
        .minor_opcode = X11_RANDR_SELECT_INPUT,
        .length = sizeof(req) / 4,
        .window = window,
        .enable = X11_RANDR_SCREEN_CHANGE_NOTIFY_MASK,
    };
    x11_write_all(conn->fd, &req, sizeof(req));
}

bool x11_wait_for_event(x11_connection_t* conn, x11_event_t* event) {
    uint8_t buf[32];
    if (!x11_read_all(conn->fd, buf, 32))
        return false;
    event->type = buf[0] & ~0x80;
    event->expose_count = 0;
    event->configure_x = 0;
    event->configure_y = 0;
    event->configure_width = 0;
    event->configure_height = 0;

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

    if (event->type == X11_CONFIGURE_NOTIFY) {
        memcpy(&event->configure_x, &buf[16], 2);
        memcpy(&event->configure_y, &buf[18], 2);
        memcpy(&event->configure_width, &buf[20], 2);
        memcpy(&event->configure_height, &buf[22], 2);
    }

    if (event->type == X11_CLIENT_MESSAGE) {
        uint32_t msg_type;
        memcpy(&msg_type, &buf[8], 4);
        if (msg_type == conn->wm_protocols) {
            uint32_t data0;
            memcpy(&data0, &buf[12], 4);
            if (data0 == conn->wm_delete_window) {
                event->type = X11_CLIENT_MESSAGE_CLOSE;
            } else if (conn->has_sync && data0 == conn->net_wm_sync_request) {
                memcpy(&event->sync_value_lo, &buf[20], 4);
                memcpy(&event->sync_value_hi, &buf[24], 4);
                event->type = X11_CLIENT_MESSAGE_SYNC_REQUEST;
            }
        }
    }

    // Translate RRScreenChangeNotify into the synthetic type
    if (conn->has_randr && conn->randr_first_event != 0 && event->type == conn->randr_first_event) {
        event->type = X11_RANDR_SCREEN_CHANGE_NOTIFY;
    }

    return true;
}

uint32_t x11_sync_create_counter(x11_connection_t* conn) {
    uint32_t id = x11_generate_id(conn);
    x11_sync_create_counter_request_t req = {
        .major_opcode = conn->sync_opcode,
        .minor_opcode = X11_SYNC_CREATE_COUNTER,
        .length = sizeof(req) / 4,
        .id = id,
        .initial_value = {.high = 0, .low = 0},
    };
    x11_write_all(conn->fd, &req, sizeof(req));
    return id;
}

void x11_sync_set_counter(x11_connection_t* conn, uint32_t counter, int32_t lo, int32_t hi) {
    x11_sync_set_counter_request_t req = {
        .major_opcode = conn->sync_opcode,
        .minor_opcode = X11_SYNC_SET_COUNTER,
        .length = sizeof(req) / 4,
        .counter = counter,
        .value = {.high = hi, .low = (uint32_t)lo},
    };
    x11_write_all(conn->fd, &req, sizeof(req));
}

void x11_sync_destroy_counter(x11_connection_t* conn, uint32_t counter) {
    x11_sync_destroy_counter_request_t req = {
        .major_opcode = conn->sync_opcode,
        .minor_opcode = X11_SYNC_DESTROY_COUNTER,
        .length = sizeof(req) / 4,
        .counter = counter,
    };
    x11_write_all(conn->fd, &req, sizeof(req));
}

void x11_disconnect(x11_connection_t* conn) {
    close(conn->fd);
}
