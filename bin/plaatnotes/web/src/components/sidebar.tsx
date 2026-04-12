/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import { type ComponentChildren } from 'preact';
import { useLocation, useRoute } from 'wouter-preact';
import { Navbar } from './navbar.tsx';

interface SidebarLayoutProps {
    children: ComponentChildren;
    sidebar: ComponentChildren;
    showSearch?: boolean;
}

export function SidebarLayout({ children, sidebar, showSearch = false }: SidebarLayoutProps) {
    return (
        <div class="min-h-screen bg-gray-50 dark:bg-zinc-900 flex flex-col">
            <Navbar showSearch={showSearch} />
            <div class="flex flex-1">
                <aside class="w-14 sm:w-56 shrink-0 flex flex-col bg-white dark:bg-zinc-800 border-r border-gray-100 dark:border-zinc-700 pt-2 sticky top-16 self-start h-[calc(100vh-4rem)] overflow-y-auto">
                    <nav class="flex flex-col gap-0.5 px-2">{sidebar}</nav>
                    <div class="flex-1" />
                    <p class="hidden sm:block px-4 pb-4 text-xs text-gray-400 dark:text-zinc-500">v{__APP_VERSION__}</p>
                </aside>
                <main class="flex-1 min-w-0">{children}</main>
            </div>
        </div>
    );
}

export function SidebarLink({ href, label, children }: { href: string; label: string; children: ComponentChildren }) {
    const [matches] = useRoute(href);
    const [, navigate] = useLocation();
    return (
        <a
            href={href}
            onClick={(e: MouseEvent) => {
                e.preventDefault();
                navigate(href);
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
    );
}
