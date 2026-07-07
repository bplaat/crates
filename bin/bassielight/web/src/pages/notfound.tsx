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
        <div class="section">
            <h2 class="title">404 Not Found</h2>
            <p class="block">No idea how you got here 🤷‍♂️</p>
        </div>
    );
}
