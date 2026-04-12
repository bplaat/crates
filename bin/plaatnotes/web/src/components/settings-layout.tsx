/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import { type ComponentChildren } from 'preact';
import { t } from '../services/i18n.service.ts';
import { AccountIcon, DownloadIcon, LaptopIcon } from './icons.tsx';
import { SidebarLayout, SidebarLink } from './sidebar.tsx';

export function SettingsLayout({ children }: { children: ComponentChildren }) {
    return (
        <SidebarLayout
            sidebar={
                <>
                    <SidebarLink href="/settings" label={t('settings.account')}>
                        <AccountIcon class="w-5 h-5 shrink-0" />
                    </SidebarLink>
                    <SidebarLink href="/settings/sessions" label={t('settings.sessions')}>
                        <LaptopIcon class="w-5 h-5 shrink-0" />
                    </SidebarLink>
                    <SidebarLink href="/settings/imports" label={t('settings.imports')}>
                        <DownloadIcon class="w-5 h-5 shrink-0" />
                    </SidebarLink>
                </>
            }
        >
            {children}
        </SidebarLayout>
    );
}
