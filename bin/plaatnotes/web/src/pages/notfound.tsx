/*
 * Copyright (c) 2025-2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import { Button, TextBoxIcon } from 'plaatui';
import { useEffect } from 'preact/hooks';
import { useLocation } from 'wouter-preact';
import { t } from '../services/i18n.service.ts';
import './notfound.css';

export function NotFound() {
    const [, navigate] = useLocation();

    useEffect(() => {
        document.title = `PlaatNotes - ${t('page.notfound')}`;
    }, []);

    return (
        <div class="notfound">
            <TextBoxIcon class="is-huge" />
            <h1 class="notfound-code">404</h1>
            <p>{t('notfound.message')}</p>
            <Button onClick={() => navigate('/')}>{t('notfound.go_home')}</Button>
        </div>
    );
}
