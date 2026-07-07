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
        <div class="notfound">
            <NoteTextIcon class="is-huge" />
            <h1 class="notfound-code">404</h1>
            <p>{t('notfound.message')}</p>
            <Button onClick={() => navigate('/')}>{t('notfound.go_home')}</Button>
        </div>
    );
}
