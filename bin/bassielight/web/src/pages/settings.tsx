/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import { useEffect } from 'preact/hooks';

export function SettingsPage() {
    useEffect(() => {
        document.title = 'BassieLight - Settings';
    }, []);

    return (
        <div class="main has-content">
            <h2 class="subtitle">Settings</h2>
            <p>
                Made by{' '}
                <a href="https://bplaat.nl/" target="_blank">
                    Bastiaan van der Plaat
                </a>{' '}
                and{' '}
                <a href="https://leonard.plaatsoft.nl/" target="_blank">
                    Leonard van der Plaat
                </a>
            </p>
        </div>
    );
}
