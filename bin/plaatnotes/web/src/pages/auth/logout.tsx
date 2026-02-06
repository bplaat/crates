/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import { useEffect } from 'preact/hooks';
import { route } from '../../router.tsx';
import { AuthService } from '../../services/auth.service.ts';

export function Logout() {
    useEffect(() => {
        document.title = 'PlaatNotes - Logging out...';

        (async () => {
            await AuthService.getInstance().logout();
            route('/auth/login');
        })();
    }, []);

    return (
        <div class="container">
            <h1 class="title">PlaatNotes</h1>
            <p>Logging out...</p>
        </div>
    );
}
