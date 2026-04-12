/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import type { ComponentChildren } from 'preact';
import { useLocation } from 'wouter-preact';
import { Nav } from './nav.tsx';

interface SidebarLayoutProps {
    children: ComponentChildren;
    sidebar: ComponentChildren;
}

export function SidebarLayout({ children, sidebar }: SidebarLayoutProps) {
    return (
        <div class="sidebar-shell">
            <Nav />
            <div class="sidebar-body">
                <aside class="sidebar-aside">
                    <nav class="sidebar-nav">{sidebar}</nav>
                </aside>
                <main class="sidebar-content">{children}</main>
            </div>
        </div>
    );
}

interface SidebarLinkProps {
    href: string;
    label: string;
    children?: ComponentChildren;
    class?: string;
}

export function SidebarLink({ href, label, children, class: extraClass }: SidebarLinkProps) {
    const [location, navigate] = useLocation();
    const isActive = location === href || (href !== '/' && location.startsWith(`${href}/`));

    return (
        <a
            href={href}
            title={label}
            class={`sidebar-link ${isActive ? 'active' : ''}${extraClass ? ` ${extraClass}` : ''}`}
            onClick={(event) => {
                event.preventDefault();
                navigate(href);
            }}
        >
            {children}
            <span class="sidebar-link-label">{label}</span>
        </a>
    );
}
