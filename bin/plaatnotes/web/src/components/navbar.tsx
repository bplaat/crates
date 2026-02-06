/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import { UserRole } from '../../src-gen/api.ts';
import { Link } from '../router.tsx';
import { $authUser } from '../services/auth.service.ts';
import { useSignal } from '@preact/signals';
import { useEffect } from 'preact/hooks';

export function Navbar() {
    const authUser = useSignal($authUser.value);

    useEffect(() => {
        const unsub = $authUser.subscribe((v) => (authUser.value = v));
        return () => unsub();
    }, []);

    return (
        <nav class="navbar is-light" role="navigation" aria-label="main navigation">
            <div class="navbar-brand">
                <Link href="/" class="navbar-item">
                    <strong>PlaatNotes</strong>
                </Link>
            </div>

            <div class="navbar-menu">
                <div class="navbar-end">
                    <div class="navbar-item">
                        <div class="buttons">
                            {authUser.value && (
                                <>
                                    <span class="navbar-item" style={{ marginRight: '1rem' }}>
                                        {authUser.value.firstName} {authUser.value.lastName}
                                    </span>
                                    {authUser.value.role === UserRole.ADMIN && (
                                        <>
                                            <Link href="/admin/users" class="button is-small is-info">
                                                <span>Users</span>
                                            </Link>
                                            <Link href="/admin/notes" class="button is-small is-info">
                                                <span>All Notes</span>
                                            </Link>
                                        </>
                                    )}
                                    <Link href="/settings" class="button is-small is-warning">
                                        <span>Settings</span>
                                    </Link>
                                    <Link href="/auth/logout" class="button is-small is-light">
                                        <span>Logout</span>
                                    </Link>
                                </>
                            )}
                        </div>
                    </div>
                </div>
            </div>
        </nav>
    );
}
