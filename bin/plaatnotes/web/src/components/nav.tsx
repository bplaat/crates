/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import { useState } from 'preact/hooks';
import { $token, $user, logout } from '../auth.ts';
import { Link, route } from '../router.tsx';

export function Nav() {
    const [dropdownOpen, setDropdownOpen] = useState(false);

    if (!$token.value) {
        return null;
    }

    async function handleLogout() {
        await logout();
        route('/login');
    }

    return (
        <nav class="navbar is-light">
            <div class="navbar-brand">
                <Link href="/" class="navbar-item">
                    <strong>PlaatNotes</strong>
                </Link>
            </div>

            <div class="navbar-menu">
                <div class="navbar-end">
                    <div class="navbar-item has-dropdown is-hoverable">
                        <a class="navbar-link">
                            {$user.value?.email || 'Account'}
                        </a>
                        <div class="navbar-dropdown is-right">
                            <Link href="/settings" class="navbar-item">
                                Settings
                            </Link>
                            <hr class="navbar-divider" />
                            <a class="navbar-item" onClick={handleLogout}>
                                Logout
                            </a>
                        </div>
                    </div>
                </div>
            </div>
        </nav>
    );
}
