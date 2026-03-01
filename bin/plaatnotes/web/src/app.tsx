/*
 * Copyright (c) 2025-2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import Router, { route } from 'preact-router';
import { useEffect } from 'preact/hooks';
import { ArchivePage } from './pages/archive.tsx';
import { AuthLogin } from './pages/auth/login.tsx';
import { Home } from './pages/home.tsx';
import { NotesCreate } from './pages/notes/create.tsx';
import { NotesShow } from './pages/notes/show.tsx';
import { NotFound } from './pages/notfound.tsx';
import { TrashPage } from './pages/trash.tsx';
import { $authUser, initAuth } from './services/auth.service.ts';

export function App() {
    // @ts-ignore
    useEffect(async () => {
        await initAuth();
    }, []);

    const authUser = $authUser.value;

    useEffect(() => {
        if (authUser === null && window.location.pathname !== '/auth/login') {
            route('/auth/login');
        }
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

    // Show nothing while auth is loading to avoid flash of unauthenticated content
    if (authUser === undefined) {
        return (
            <div class="min-h-screen bg-gray-50 dark:bg-zinc-900 flex items-center justify-center">
                <div class="text-gray-400 dark:text-gray-500 text-sm">Loading…</div>
            </div>
        );
    }

    return (
        <Router>
            <AuthLogin path="/auth/login" />
            <Home path="/" />
            <ArchivePage path="/archive" />
            <TrashPage path="/trash" />
            <NotesCreate path="/notes/create" />
            <NotesShow path="/notes/:note_id" />
            <NotFound default />
        </Router>
    );
}
