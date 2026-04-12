/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import { type ComponentChildren } from 'preact';
import { t } from '../services/i18n.service.ts';
import { AccountMultipleIcon } from './icons.tsx';
import { SidebarLayout, SidebarLink } from './sidebar.tsx';

export function AdminLayout({ children }: { children: ComponentChildren }) {
    return (
        <SidebarLayout
            sidebar={
                <SidebarLink href="/admin/users" label={t('admin.users.sidebar')}>
                    <AccountMultipleIcon class="w-5 h-5 shrink-0" />
                </SidebarLink>
            }
        >
            {children}
        </SidebarLayout>
    );
}
