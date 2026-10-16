/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import { AccountMultipleIcon, SidebarLayout } from 'plaatui';
import { type ComponentChildren } from 'preact';
import { t } from '../services/i18n.service.ts';
import { PlaatNotesNavbar } from './navbar.tsx';
import { SidebarLink } from './sidebar-link.tsx';

export function AdminLayout({ children }: { children: ComponentChildren }) {
    return (
        <SidebarLayout
            navbar={<PlaatNotesNavbar />}
            version={__APP_VERSION__}
            sidebar={<SidebarLink href="/admin/users" label={t('admin.users.sidebar')} icon={AccountMultipleIcon} />}
        >
            {children}
        </SidebarLayout>
    );
}
