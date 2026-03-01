/*
 * Copyright (c) 2025-2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import Router from 'preact-router';
import { useEffect } from 'preact/hooks';
import { ArchivePage } from './pages/archive.tsx';
import { AuthLogin } from './pages/auth/login.tsx';
import { Home } from './pages/home.tsx';
import { NotesCreate } from './pages/notes/create.tsx';
import { NotesShow } from './pages/notes/show.tsx';
import { NotFound } from './pages/notfound.tsx';
import { Settings } from './pages/settings.tsx';
import { TrashPage } from './pages/trash.tsx';
import { $authUser, initAuth } from './services/auth.service.ts';
import { setLanguage, t } from './services/i18n.service.ts';

export function App() {
    // @ts-ignore
    useEffect(async () => {
        await initAuth();
    }, []);

    const authUser = $authUser.value;

    useEffect(() => {
        if (authUser) setLanguage(authUser.language);
    }, [authUser]);

    useEffect(() => {
        function applyTheme() {
            const theme = authUser?.theme ?? 'system';
            const isDark =
                theme === 'dark' || (theme === 'system' && window.matchMedia('(prefers-color-scheme: dark)').matches);
            document.documentElement.classList.toggle('dark', isDark);
        }
        applyTheme();
        const theme = authUser?.theme ?? 'system';
        if (theme === 'system') {
            const mq = window.matchMedia('(prefers-color-scheme: dark)');
            mq.addEventListener('change', applyTheme);
            return () => mq.removeEventListener('change', applyTheme);
        }
    }, [authUser]);

    // Show loading screen while auth resolves
    if (authUser === undefined) {
        return (
            <div class="min-h-screen bg-gray-50 dark:bg-zinc-900 flex items-center justify-center">
                <div class="text-gray-400 dark:text-gray-500 text-sm">{t('app.loading')}</div>
            </div>
        );
    }

    // Render login directly when unauthenticated so no authenticated page ever
    // renders with a null user (e.g. after logout while on a protected page)
    if (authUser === null) {
        return <AuthLogin />;
    }

    return (
        <Router>
            <Home path="/" />
            <ArchivePage path="/archive" />
            <TrashPage path="/trash" />
            <NotesCreate path="/notes/create" />
            <NotesShow path="/notes/:note_id" />
            <Settings path="/settings" />
            <NotFound default />
        </Router>
    );
}
