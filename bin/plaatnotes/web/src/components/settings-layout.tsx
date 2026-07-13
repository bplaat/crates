/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import { AccountIcon, DownloadIcon, LaptopIcon, SidebarLayout } from 'plaatui';
import { type ComponentChildren } from 'preact';
import { t } from '../services/i18n.service.ts';
import { PlaatNotesNavbar } from './navbar.tsx';
import { SidebarLink } from './sidebar-link.tsx';

export function SettingsLayout({ children }: { children: ComponentChildren }) {
    return (
        <SidebarLayout
            navbar={<PlaatNotesNavbar />}
            version={__APP_VERSION__}
            sidebar={
                <>
                    <SidebarLink href="/settings" label={t('settings.account')} icon={AccountIcon} />
                    <SidebarLink href="/settings/sessions" label={t('settings.sessions')} icon={LaptopIcon} />
                    <SidebarLink href="/settings/imports" label={t('settings.imports')} icon={DownloadIcon} />
                </>
            }
        >
            {children}
        </SidebarLayout>
    );
}
