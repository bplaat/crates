/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import tailwindcss from '@tailwindcss/vite';
import preact from '@preact/preset-vite';
import { defineConfig } from 'vite';

export default defineConfig({
    plugins: [tailwindcss(), preact()],
    build: {
        outDir: 'dist',
    },
    server: {
        proxy: {
            '/api': 'http://localhost:8080',
        },
    },
});
