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

export function SettingsLayout({ children }: { children: ComponentChildren }) {
    return (
        <SidebarLayout
            navbar={<PlaatNotesNavbar />}
            version={__APP_VERSION__}
            sidebar={
                <>
                    <SidebarLink href="/settings" label={t('settings.account')} icon="account" />
                    <SidebarLink href="/settings/sessions" label={t('settings.sessions')} icon="laptop" />
                    <SidebarLink href="/settings/imports" label={t('settings.imports')} icon="download" />
                </>
            }
        >
            {children}
        </SidebarLayout>
    );
}
