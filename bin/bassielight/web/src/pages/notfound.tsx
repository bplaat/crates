/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import { useEffect } from 'preact/hooks';

export function NotFoundPage() {
    useEffect(() => {
        document.title = 'BassieLight - Not Found';
    }, []);

    return (
        <div class="flex-1 p-8 overflow-y-auto">
            <h2 class="font-bold mt-0 mb-4 text-[1.35rem] border-b border-zinc-600 pb-2">404 Not Found</h2>
            <p class="my-4">No idea how you got here 🤷‍♂️</p>
        </div>
    );
}
