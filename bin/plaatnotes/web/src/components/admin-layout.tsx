/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import { type ComponentChildren } from 'preact';
import Match from 'preact-router/match';
import { route } from 'preact-router';
import { Navbar } from './navbar.tsx';
import { t } from '../services/i18n.service.ts';

function AdminSidebarLink({ href, label, children }: { href: string; label: string; children: ComponentChildren }) {
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

export function AdminLayout({ children }: { children: ComponentChildren }) {
    return (
        <div class="h-screen overflow-hidden bg-gray-50 dark:bg-zinc-900 flex flex-col">
            <Navbar />
            <div class="flex flex-1 overflow-hidden">
                <aside class="w-14 sm:w-56 shrink-0 bg-white dark:bg-zinc-800 border-r border-gray-100 dark:border-zinc-700 pt-2 pb-4 overflow-y-auto">
                    <nav class="flex flex-col gap-0.5 px-2">
                        <AdminSidebarLink href="/admin/users" label={t('admin.users.sidebar')}>
                            <svg class="w-5 h-5 shrink-0" viewBox="0 0 24 24" fill="currentColor">
                                <path d="M16 11c1.66 0 2.99-1.34 2.99-3S17.66 5 16 5c-1.66 0-3 1.34-3 3s1.34 3 3 3zm-8 0c1.66 0 2.99-1.34 2.99-3S9.66 5 8 5C6.34 5 5 6.34 5 8s1.34 3 3 3zm0 2c-2.33 0-7 1.17-7 3.5V19h14v-2.5c0-2.33-4.67-3.5-7-3.5zm8 0c-.29 0-.62.02-.97.05 1.16.84 1.97 1.97 1.97 3.45V19h6v-2.5c0-2.33-4.67-3.5-7-3.5z" />
                            </svg>
                        </AdminSidebarLink>
                    </nav>
                </aside>
                <main class="flex-1 min-w-0 overflow-y-auto">{children}</main>
            </div>
        </div>
    );
}
