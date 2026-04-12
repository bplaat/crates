/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import { useEffect, useRef, useState } from 'preact/hooks';
import { Link, useLocation } from 'wouter-preact';
import { $authUser, logout } from '../services/auth.ts';

export function Nav() {
    const user = $authUser.value;
    const [menuOpen, setMenuOpen] = useState(false);
    const menuRef = useRef<HTMLDivElement>(null);
    const [, navigate] = useLocation();

    useEffect(() => {
        function handleDocumentClick(event: MouseEvent) {
            if (menuRef.current && !menuRef.current.contains(event.target as Node)) {
                setMenuOpen(false);
            }
        }

        if (menuOpen) {
            document.addEventListener('mousedown', handleDocumentClick);
            return () => document.removeEventListener('mousedown', handleDocumentClick);
        }
    }, [menuOpen]);

    const displayName = [user?.firstName, user?.lastName].filter(Boolean).join(' ').trim() || user?.email || 'User';
    const initials = `${user?.firstName?.[0] ?? ''}${user?.lastName?.[0] ?? ''}`.toUpperCase() || 'U';

    return (
        <nav>
            <Link href="/" class="brand">
                PlaatDeploy
            </Link>
            <Link href="/">Home</Link>
            <Link href="/teams">Teams</Link>
            <span class="spacer" />
            <div class="nav-user-menu" ref={menuRef}>
                <button class="nav-user-trigger" onClick={() => setMenuOpen((open) => !open)}>
                    <span class="nav-user-avatar">{initials}</span>
                    <span class="nav-user-name">{displayName}</span>
                </button>
                {menuOpen && (
                    <div class="nav-user-dropdown">
                        {user?.role === 'admin' && (
                            <button
                                class="nav-user-dropdown-item"
                                onClick={() => {
                                    setMenuOpen(false);
                                    navigate('/admin/users');
                                }}
                            >
                                Admin
                            </button>
                        )}
                        <button
                            class="nav-user-dropdown-item"
                            onClick={() => {
                                setMenuOpen(false);
                                navigate('/settings');
                            }}
                        >
                            Settings
                        </button>
                        <button
                            class="nav-user-dropdown-item"
                            onClick={async () => {
                                setMenuOpen(false);
                                await logout();
                            }}
                        >
                            Logout
                        </button>
                    </div>
                )}
            </div>
        </nav>
    );
}
