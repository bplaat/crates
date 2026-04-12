/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import { type ComponentChildren } from 'preact';
import { t } from '../services/i18n.service.ts';
import { SidebarLayout, SidebarLink } from './sidebar.tsx';

export function AdminLayout({ children }: { children: ComponentChildren }) {
    return (
        <SidebarLayout
            sidebar={
                <SidebarLink href="/admin/users" label={t('admin.users.sidebar')}>
                    <svg class="w-5 h-5 shrink-0" viewBox="0 0 24 24" fill="currentColor">
                        <path d="M16 11c1.66 0 2.99-1.34 2.99-3S17.66 5 16 5c-1.66 0-3 1.34-3 3s1.34 3 3 3zm-8 0c1.66 0 2.99-1.34 2.99-3S9.66 5 8 5C6.34 5 5 6.34 5 8s1.34 3 3 3zm0 2c-2.33 0-7 1.17-7 3.5V19h14v-2.5c0-2.33-4.67-3.5-7-3.5zm8 0c-.29 0-.62.02-.97.05 1.16.84 1.97 1.97 1.97 3.45V19h6v-2.5c0-2.33-4.67-3.5-7-3.5z" />
                    </svg>
                </SidebarLink>
            }
        >
            {children}
        </SidebarLayout>
    );
}
