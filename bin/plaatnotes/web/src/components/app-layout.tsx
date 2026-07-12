/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import { type ComponentChildren } from 'preact';
import { t } from '../services/i18n.service.ts';
import { SidebarLayout } from 'plaatui';
import { PlaatNotesNavbar } from './navbar.tsx';
import { SidebarLink } from './sidebar-link.tsx';

export function AppLayout({ children, showSearch }: { children: ComponentChildren; showSearch?: boolean }) {
    return (
        <SidebarLayout
            navbar={<PlaatNotesNavbar showSearch={showSearch} />}
            version={__APP_VERSION__}
            sidebar={
                <>
                    <SidebarLink href="/" label={t('sidebar.notes')} icon="text-box" />
                    <SidebarLink href="/archive" label={t('sidebar.archive')} icon="package-down" />
                    <SidebarLink href="/trash" label={t('sidebar.trash')} icon="delete" />
                </>
            }
        >
            {children}
        </SidebarLayout>
    );
}
