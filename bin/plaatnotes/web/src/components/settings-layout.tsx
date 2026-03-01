/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import { route } from 'preact-router';
import { Navbar } from './navbar.tsx';
import { t } from '../services/i18n.service.ts';

function SettingsSidebarLink({
    href,
    label,
    children,
}: {
    href: string;
    label: string;
    children: any;
}) {
    const active = window.location.pathname === href;
    return (
        <a
            href={href}
            onClick={(e: MouseEvent) => {
                e.preventDefault();
                route(href);
            }}
            title={label}
            class={`flex items-center gap-3 px-3 py-2.5 rounded-full transition-colors no-underline ${
                active
                    ? 'bg-yellow-50 text-yellow-700 dark:bg-yellow-900/30 dark:text-yellow-400'
                    : 'text-gray-600 hover:bg-gray-100 dark:text-gray-300 dark:hover:bg-zinc-700'
            }`}
        >
            {children}
            <span class="hidden sm:block text-sm font-medium">{label}</span>
        </a>
    );
}

export function SettingsLayout({ children }: { children: any }) {
    return (
        <div class="min-h-screen bg-gray-50 dark:bg-zinc-900 flex flex-col">
            <Navbar />
            <div class="flex flex-1">
                <aside class="w-14 sm:w-56 shrink-0 bg-white dark:bg-zinc-800 border-r border-gray-100 dark:border-zinc-700 pt-2 pb-4">
                    <nav class="flex flex-col gap-0.5 px-2">
                        <SettingsSidebarLink href="/settings" label={t('settings.account')}>
                            <svg class="w-5 h-5 shrink-0" viewBox="0 0 24 24" fill="currentColor">
                                <path d="M12 12c2.21 0 4-1.79 4-4s-1.79-4-4-4-4 1.79-4 4 1.79 4 4 4zm0 2c-2.67 0-8 1.34-8 4v2h16v-2c0-2.66-5.33-4-8-4z" />
                            </svg>
                        </SettingsSidebarLink>
                        <SettingsSidebarLink href="/settings/sessions" label={t('settings.sessions')}>
                            <svg class="w-5 h-5 shrink-0" viewBox="0 0 24 24" fill="currentColor">
                                <path d="M4 6h18V4H4c-1.1 0-2 .9-2 2v11H0v3h14v-3H4V6zm19 2h-6c-.55 0-1 .45-1 1v10c0 .55.45 1 1 1h6c.55 0 1-.45 1-1V9c0-.55-.45-1-1-1zm-1 9h-4v-7h4v7z" />
                            </svg>
                        </SettingsSidebarLink>
                    </nav>
                </aside>
                <main class="flex-1 min-w-0">{children}</main>
            </div>
        </div>
    );
}
