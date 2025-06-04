/*
 * Copyright (c) 2023, Bastiaan van der Plaat <bastiaan.v.d.plaat@gmail.com>
 *
 * SPDX-License-Identifier: MIT
 */

const SERVER_PORT = 8080;

const MessageType = {
    BOARD_COLORS: 0x00,
    BOARD_LABELS: 0x01,
    BUTTON_PRESS: 0x10,
    BUTTON_RELEASE: 0x11,
};

const COLORS = [
    { name: 'Off', red: 0, green: 0, blue: 0 },
    { name: 'Blue', red: 0, green: 0, blue: 170 },
    { name: 'Green', red: 0, green: 170, blue: 0 },
    { name: 'Cyan', red: 0, green: 170, blue: 170 },
    { name: 'Red', red: 170, green: 0, blue: 0 },
    { name: 'Magenta', red: 170, green: 0, blue: 170 },
    { name: 'Brown', red: 170, green: 85, blue: 0 },
    { name: 'Light Gray', red: 170, green: 170, blue: 170 },
    { name: 'Dark Gray', red: 85, green: 85, blue: 85 },
    { name: 'Light Blue', red: 85, green: 85, blue: 255 },
    { name: 'Light Green', red: 85, green: 255, blue: 85 },
    { name: 'Light Cyan', red: 85, green: 255, blue: 255 },
    { name: 'Light Red', red: 255, green: 85, blue: 85 },
    { name: 'Light Magenta', red: 255, green: 85, blue: 255 },
    { name: 'Yellow', red: 255, green: 255, blue: 85 },
    { name: 'White', red: 255, green: 255, blue: 255 },
];

const SPEEDS = [0, 1000, 750, 500, 250, 100, 50, 22.4];

export { SERVER_PORT, MessageType, COLORS, SPEEDS };
