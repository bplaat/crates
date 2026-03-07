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
        <div class="flex-1 p-8 overflow-y-auto">
            <h2 class="font-bold mt-0 mb-4 text-[1.35rem] border-b border-zinc-600 pb-2">Settings</h2>
            <p class="my-4">
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
