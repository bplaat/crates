/*
 * Copyright (c) 2025-2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import { createRequire } from 'node:module';
import { dirname, join } from 'node:path';
import preact from '@preact/preset-vite';
import { defineConfig } from 'vite';

// The monaco-editor package only exports its scripts, its stylesheets are resolved from the package
const monaco = join(dirname(createRequire(import.meta.url).resolve('monaco-editor/editor/editor.api.js')), '..');

export default defineConfig({
    plugins: [preact()],
    resolve: {
        alias: [{ find: /^monaco-editor\/(.*\.css)$/, replacement: `${monaco}/$1` }],
    },
});
