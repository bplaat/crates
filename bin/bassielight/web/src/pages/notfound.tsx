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
        <div class="main has-content">
            <h2 class="subtitle">404 Not Found</h2>
            <p>No idea how you got here 🤷‍♂️</p>
        </div>
    );
}
