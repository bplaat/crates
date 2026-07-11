/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import type { ComponentChildren } from 'preact';
import { SidebarLayout, SidebarLink } from './sidebar-layout.tsx';
import { AccountMultipleIcon, AccountIcon } from './icons.tsx';

export function AdminLayout({ children }: { children: ComponentChildren }) {
    return (
        <SidebarLayout
            sidebar={
                <>
                    <SidebarLink href="/admin/users" label="Users">
                        <AccountIcon class="sidebar-link-icon" />
                    </SidebarLink>
                    <SidebarLink href="/admin/teams" label="Teams">
                        <AccountMultipleIcon class="sidebar-link-icon" />
                    </SidebarLink>
                </>
            }
        >
            {children}
        </SidebarLayout>
    );
}
