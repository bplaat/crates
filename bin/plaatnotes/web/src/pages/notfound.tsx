/*
 * Copyright (c) 2025-2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import { useLocation } from 'wouter-preact';
import { useEffect } from 'preact/hooks';
import { Button } from '../components/button.tsx';
import { NoteTextIcon } from '../components/icons.tsx';
import { t } from '../services/i18n.service.ts';

export function NotFound() {
    const [, navigate] = useLocation();

    useEffect(() => {
        document.title = `PlaatNotes - ${t('page.notfound')}`;
    }, []);

    return (
        <div class="min-h-screen bg-gray-50 dark:bg-zinc-900 flex flex-col items-center justify-center gap-4 text-gray-400 dark:text-gray-500">
            <NoteTextIcon class="w-16 h-16" />
            <h1 class="text-4xl font-light text-gray-500 dark:text-gray-400">404</h1>
            <p class="text-gray-400 dark:text-gray-500">{t('notfound.message')}</p>
            <Button class="mt-2" onClick={() => navigate('/')}>
                {t('notfound.go_home')}
            </Button>
        </div>
    );
}
