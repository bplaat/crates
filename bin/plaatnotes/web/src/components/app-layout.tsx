/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import { DeleteIcon, PackageDownIcon, SidebarLayout, TextBoxIcon } from 'plaatui';
import { type ComponentChildren } from 'preact';
import { t } from '../services/i18n.service.ts';
import { PlaatNotesNavbar } from './navbar.tsx';
import { SidebarLink } from './sidebar-link.tsx';

export function AppLayout({ children, showSearch }: { children: ComponentChildren; showSearch?: boolean }) {
    return (
        <SidebarLayout
            navbar={<PlaatNotesNavbar showSearch={showSearch} />}
            version={__APP_VERSION__}
            sidebar={
                <>
                    <SidebarLink href="/" label={t('sidebar.notes')} icon={TextBoxIcon} />
                    <SidebarLink href="/archive" label={t('sidebar.archive')} icon={PackageDownIcon} />
                    <SidebarLink href="/trash" label={t('sidebar.trash')} icon={DeleteIcon} />
                </>
            }
        >
            {children}
        </SidebarLayout>
    );
}
