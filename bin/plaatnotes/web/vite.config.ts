/*
 * Copyright (c) 2025-2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import { readFileSync } from 'node:fs';
import tailwindcss from '@tailwindcss/vite';
import { defineConfig } from 'vite';
import preact from '@preact/preset-vite';

const { version } = JSON.parse(readFileSync('./package.json', 'utf-8'));

export default defineConfig({
    plugins: [preact(), tailwindcss()],
    define: {
        __APP_VERSION__: JSON.stringify(version),
    },
});
