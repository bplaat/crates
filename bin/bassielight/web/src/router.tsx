/*
 * Copyright (c) 2024-2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import { signal, useSignal } from '@preact/signals';
import { useLiveSignal } from '@preact/signals/utils';
import { useEffect } from 'preact/hooks';

export const $route = signal(window.location.pathname);
let matches = false;

// MARK: Components
export function Router({ children }: { children: any }) {
    // Listen to popstate event for back navigation
    useEffect(() => {
        const handlePopState = () => {
            $route.value = new URL(window.location.href).pathname;
            matches = false;
        };
        window.addEventListener('popstate', handlePopState);
        return () => window.removeEventListener('popstate', handlePopState);
    }, []);
    return children;
}

export function Route({ path, component, fallback }: { path?: string; fallback?: boolean; component: any }) {
    const Component = component;
    const route = $route.value;

    if (fallback) {
        if (!matches) {
            matches = true;
            return <Component />;
        }
        return null;
    }

    const paramNames = path!.match(/:([^/]+)/g) || [];
    const match = route.match(new RegExp(`^${path!.replace(/:([^/]+)/g, '([^/]+)')}$`));
    if (match && !matches) {
        const params: { [key: string]: string } = {};
        for (let i = 0; i < paramNames.length; i++) {
            params[paramNames[i].substring(1)] = match[i + 1];
        }
        matches = true;
        return <Component {...params} />;
    }
    return null;
}

export function Link({ href, ...props }: { href: string; [key: string]: any }) {
    const open = (event: MouseEvent) => {
        event.preventDefault();
        event.stopPropagation();
        route(href);
    };
    return <a {...props} href={href} onClick={open} />;
}

// MARK: Utils
export function route(to: string) {
    window.history.pushState({}, '', to);
    window.scrollTo(0, 0);
    $route.value = new URL(to, window.location.origin).pathname;
    matches = false;
}
