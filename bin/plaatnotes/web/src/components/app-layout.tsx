/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import { type ComponentChildren } from 'preact';
import { t } from '../services/i18n.service.ts';
import { ArchiveArrowDownIcon, DeleteOutlineIcon, NoteTextIcon } from './icons.tsx';
import { SidebarLayout, SidebarLink } from './sidebar.tsx';

export function AppLayout({ children, showSearch }: { children: ComponentChildren; showSearch?: boolean }) {
    return (
        <SidebarLayout
            showSearch={showSearch}
            sidebar={
                <>
                    <SidebarLink href="/" label={t('sidebar.notes')}>
                        <NoteTextIcon class="w-5 h-5 shrink-0" />
                    </SidebarLink>
                    <SidebarLink href="/archive" label={t('sidebar.archive')}>
                        <ArchiveArrowDownIcon class="w-5 h-5 shrink-0" />
                    </SidebarLink>
                    <SidebarLink href="/trash" label={t('sidebar.trash')}>
                        <DeleteOutlineIcon class="w-5 h-5 shrink-0" />
                    </SidebarLink>
                </>
            }
        >
            {children}
        </SidebarLayout>
    );
}
