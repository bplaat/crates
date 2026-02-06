/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import type { UserTheme } from '../../src-gen/api.ts';

export class ThemeService {
    static applyTheme(theme: UserTheme): void {
        const body = document.body;

        if (theme === 'system') {
            body.classList.remove('is-light', 'is-dark');
            // Apply system preference if available
            const prefersLight = window.matchMedia('(prefers-color-scheme: light)').matches;
            if (prefersLight) {
                body.classList.add('is-light');
            } else {
                body.classList.add('is-dark');
            }
        } else if (theme === 'light') {
            body.classList.remove('is-dark');
            body.classList.add('is-light');
        } else if (theme === 'dark') {
            body.classList.remove('is-light');
            body.classList.add('is-dark');
        }
    }

    static watchSystemTheme(): () => void {
        const mediaQuery = window.matchMedia('(prefers-color-scheme: light)');
        const listener = (e: MediaQueryListEvent) => {
            const body = document.body;
            // Only apply if user theme is system (no explicit light/dark classes)
            if (!body.classList.contains('is-light') && !body.classList.contains('is-dark')) {
                if (e.matches) {
                    body.classList.add('is-light');
                    body.classList.remove('is-dark');
                } else {
                    body.classList.add('is-dark');
                    body.classList.remove('is-light');
                }
            }
        };
        mediaQuery.addEventListener('change', listener);
        return () => mediaQuery.removeEventListener('change', listener);
    }
}
