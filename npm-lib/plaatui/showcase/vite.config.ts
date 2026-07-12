/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import { readFileSync } from 'node:fs';
import { defineConfig } from 'vite';
import preact from '@preact/preset-vite';

const { version } = JSON.parse(readFileSync('./package.json', 'utf-8'));

export default defineConfig({
    base: '/plaatui/',
    plugins: [preact()],
    define: {
        __APP_VERSION__: JSON.stringify(version),
    },
});
