/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import type { ComponentChildren } from 'preact';
import { SidebarLayout, SidebarLink } from './sidebar-layout.tsx';

export function AdminLayout({ children }: { children: ComponentChildren }) {
    return (
        <SidebarLayout
            sidebar={
                <>
                    <SidebarLink href="/admin/users" label="Users">
                        <svg class="sidebar-link-icon" viewBox="0 0 24 24" fill="currentColor">
                            <path d="M16 11c1.66 0 2.99-1.34 2.99-3S17.66 5 16 5c-1.66 0-3 1.34-3 3s1.34 3 3 3zm-8 0c1.66 0 2.99-1.34 2.99-3S9.66 5 8 5C6.34 5 5 6.34 5 8s1.34 3 3 3zm0 2c-2.33 0-7 1.17-7 3.5V19h14v-2.5c0-2.33-4.67-3.5-7-3.5zm8 0c-.29 0-.62.02-.97.05 1.16.84 1.97 1.97 1.97 3.45V19h6v-2.5c0-2.33-4.67-3.5-7-3.5z" />
                        </svg>
                    </SidebarLink>
                    <SidebarLink href="/admin/teams" label="Teams">
                        <svg class="sidebar-link-icon" viewBox="0 0 24 24" fill="currentColor">
                            <path d="M12 2a5 5 0 0 0-5 5c0 1.93 1.09 3.6 2.69 4.44A7.002 7.002 0 0 0 5 18v2h14v-2a7.002 7.002 0 0 0-4.69-6.56A4.988 4.988 0 0 0 17 7a5 5 0 0 0-5-5zm0 2a3 3 0 1 1 0 6 3 3 0 0 1 0-6zm0 8c2.97 0 5.43 2.16 5.91 5H6.09c.48-2.84 2.94-5 5.91-5z" />
                        </svg>
                    </SidebarLink>
                </>
            }
        >
            {children}
        </SidebarLayout>
    );
}
