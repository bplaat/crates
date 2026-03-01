/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import Match from 'preact-router/match';
import { route } from 'preact-router';
import { t } from '../services/i18n.service.ts';
import { Navbar } from './navbar.tsx';

function SidebarLink({ href, label, children }: { href: string; exact?: boolean; label: string; children: any }) {
    return (
        <Match path={href}>
            {({ matches }: { matches: boolean }) => (
                <a
                    href={href}
                    onClick={(e: MouseEvent) => {
                        e.preventDefault();
                        route(href);
                    }}
                    title={label}
                    class={`flex items-center gap-3 px-3 py-2.5 rounded-full transition-colors no-underline ${
                        matches
                            ? 'bg-yellow-50 text-yellow-700 dark:bg-yellow-900/30 dark:text-yellow-400'
                            : 'text-gray-600 hover:bg-gray-100 dark:text-gray-300 dark:hover:bg-zinc-700'
                    }`}
                >
                    {children}
                    <span class="hidden sm:block text-sm font-medium">{label}</span>
                </a>
            )}
        </Match>
    );
}

export function Layout({ children }: { children: any }) {
    return (
        <div class="h-screen overflow-hidden bg-gray-50 dark:bg-zinc-900 flex flex-col">
            <Navbar />
            <div class="flex flex-1 overflow-hidden">
                <aside class="w-14 sm:w-56 shrink-0 flex flex-col bg-white dark:bg-zinc-800 border-r border-gray-100 dark:border-zinc-700 pt-2 overflow-y-auto">
                    <nav class="flex flex-col gap-0.5 px-2">
                        <SidebarLink href="/" exact label={t('sidebar.notes')}>
                            <svg class="w-5 h-5 shrink-0" viewBox="0 0 24 24" fill="currentColor">
                                <path d="M19 3H5c-1.1 0-2 .9-2 2v14c0 1.1.9 2 2 2h14c1.1 0 2-.9 2-2V5c0-1.1-.9-2-2-2zm-5 14H7v-2h7v2zm3-4H7v-2h10v2zm0-4H7V7h10v2z" />
                            </svg>
                        </SidebarLink>
                        <SidebarLink href="/archive" label={t('sidebar.archive')}>
                            <svg class="w-5 h-5 shrink-0" viewBox="0 0 24 24" fill="currentColor">
                                <path d="M20.54 5.23l-1.39-1.68C18.88 3.21 18.47 3 18 3H6c-.47 0-.88.21-1.16.55L3.46 5.23C3.17 5.57 3 6.02 3 6.5V19c0 1.1.9 2 2 2h14c1.1 0 2-.9 2-2V6.5c0-.48-.17-.93-.46-1.27zM6.24 5h11.52l.83 1H5.42l.82-1zM5 19V8h14v11H5zm8.45-9h-2.9v3H8l4 4 4-4h-2.55v-3z" />
                            </svg>
                        </SidebarLink>
                        <SidebarLink href="/trash" label={t('sidebar.trash')}>
                            <svg class="w-5 h-5 shrink-0" viewBox="0 0 24 24" fill="currentColor">
                                <path d="M6 19c0 1.1.9 2 2 2h8c1.1 0 2-.9 2-2V7H6v12zM8 9h8v10H8V9zm7.5-5l-1-1h-5l-1 1H5v2h14V4h-3.5z" />
                            </svg>
                        </SidebarLink>
                    </nav>
                    <div class="flex-1" />
                    <p class="hidden sm:block px-4 pb-4 text-xs text-gray-400 dark:text-zinc-500">v{__APP_VERSION__}</p>
                </aside>
                <main class="flex-1 min-w-0 overflow-y-auto">{children}</main>
            </div>
        </div>
    );
}
