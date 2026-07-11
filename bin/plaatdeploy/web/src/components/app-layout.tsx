/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import type { ComponentChildren } from 'preact';
import { Nav } from './nav.tsx';

export function AppLayout({ children }: { children: ComponentChildren }) {
    return (
        <div class="app-shell">
            <Nav />
            <main>{children}</main>
        </div>
    );
}
