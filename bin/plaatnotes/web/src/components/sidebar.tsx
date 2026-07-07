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
        <div class="layout">
            <Navbar showSearch={showSearch} />
            <div class="layout-body">
                <aside class="sidebar">
                    <nav class="sidebar-nav">{sidebar}</nav>
                    <div class="spacer" />
                    <p class="sidebar-version">v{__APP_VERSION__}</p>
                </aside>
                <main class="layout-main">{children}</main>
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
            class={`sidebar-item ${matches ? 'is-active' : ''}`}
        >
            {children}
            <span class="sidebar-label">{label}</span>
        </a>
    );
}
