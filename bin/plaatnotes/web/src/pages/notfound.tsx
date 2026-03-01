/*
 * Copyright (c) 2025-2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import { route } from 'preact-router';
import { useEffect } from 'preact/hooks';

export function NotFound() {
    useEffect(() => {
        document.title = 'PlaatNotes - Not Found';
    }, []);

    return (
        <div class="min-h-screen bg-gray-50 flex flex-col items-center justify-center gap-4 text-gray-400">
            <svg class="w-20 h-20" viewBox="0 0 24 24" fill="currentColor">
                <path d="M19 3H5c-1.1 0-2 .9-2 2v14c0 1.1.9 2 2 2h14c1.1 0 2-.9 2-2V5c0-1.1-.9-2-2-2zm-5 14H7v-2h7v2zm3-4H7v-2h10v2zm0-4H7V7h10v2z" />
            </svg>
            <h1 class="text-4xl font-light text-gray-500">404</h1>
            <p class="text-gray-400">No idea how you got here 🤷‍♂️</p>
            <button
                onClick={() => route('/')}
                class="mt-2 px-5 py-2 bg-yellow-400 hover:bg-yellow-500 text-white rounded-lg transition-colors cursor-pointer"
            >
                Go home
            </button>
        </div>
    );
}
