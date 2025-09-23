/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import { defineConfig } from 'vite';
import inlineCssModules from 'vite-plugin-inline-css-modules';
import preact from '@preact/preset-vite';

export default defineConfig({
    plugins: [inlineCssModules(), preact()],
});
