/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import { useEffect } from 'preact/hooks';

export function NotFound() {
    useEffect(() => {
        document.title = 'PlaatNotes - Not Found';
    }, []);

    return (
        <div class="container">
            <h1 class="title">404 Not Found</h1>
            <p>No idea how you got here 🤷‍♂️</p>
        </div>
    );
}
