/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import type { ComponentChildren } from 'preact';
import { SidebarLayout, SidebarLink } from './sidebar-layout.tsx';
import { CogIcon, LaptopIcon } from './icons.tsx';

export function SettingsLayout({ children }: { children: ComponentChildren }) {
    return (
        <SidebarLayout
            sidebar={
                <>
                    <SidebarLink href="/settings" label="Settings">
                        <CogIcon class="sidebar-link-icon" />
                    </SidebarLink>
                    <SidebarLink href="/settings/sessions" label="Sessions">
                        <LaptopIcon class="sidebar-link-icon" />
                    </SidebarLink>
                </>
            }
        >
            {children}
        </SidebarLayout>
    );
}
