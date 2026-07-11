/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import { useEffect, useRef, useState } from 'preact/hooks';
import { Link, useLocation } from 'wouter-preact';
import { $authUser, logout } from '../services/auth.ts';
import { API_URL, authFetch } from '../services/auth.ts';
import { $currentTeamId, setCurrentTeamId } from '../services/current-team.ts';
import { jsonOrThrow } from '../services/api.ts';
import type { Team, TeamIndexResponse } from '../src-gen/api.ts';
import { AccountMultipleIcon, CogIcon, HomeIcon, LogoutIcon, ShieldIcon } from './icons.tsx';

export function Nav() {
    const user = $authUser.value;
    const [menuOpen, setMenuOpen] = useState(false);
    const [teams, setTeams] = useState<Team[]>([]);
    const menuRef = useRef<HTMLDivElement>(null);
    const [location, navigate] = useLocation();

    useEffect(() => {
        function handleDocumentClick(event: MouseEvent) {
            if (menuRef.current && !menuRef.current.contains(event.target as Node)) {
                setMenuOpen(false);
            }
        }

        function handleKeyDown(event: KeyboardEvent) {
            if (event.key === 'Escape') {
                setMenuOpen(false);
            }
        }

        if (menuOpen) {
            document.addEventListener('mousedown', handleDocumentClick);
            document.addEventListener('keydown', handleKeyDown);
            return () => {
                document.removeEventListener('mousedown', handleDocumentClick);
                document.removeEventListener('keydown', handleKeyDown);
            };
        }
    }, [menuOpen]);

    useEffect(() => {
        let ignore = false;
        authFetch(`${API_URL}/teams`)
            .then((response) => jsonOrThrow<TeamIndexResponse>(response))
            .then((data) => {
                if (ignore) return;
                setTeams(data.data);
                if (!data.data.some((team) => team.id === $currentTeamId.value)) {
                    setCurrentTeamId(data.data[0]?.id ?? null);
                }
            })
            .catch(() => {});
        return () => {
            ignore = true;
        };
    }, []);

    const displayName = [user?.firstName, user?.lastName].filter(Boolean).join(' ').trim() || user?.email || 'User';
    const initials = `${user?.firstName?.[0] ?? ''}${user?.lastName?.[0] ?? ''}`.toUpperCase() || 'U';

    return (
        <nav class="navbar">
            <Link href="/" class="navbar-brand">
                PlaatDeploy
            </Link>
            <Link href="/" class="navbar-item">
                <HomeIcon class="is-sm" />
                Home
            </Link>
            <Link href="/teams" class="navbar-item">
                <AccountMultipleIcon class="is-sm" />
                Team
            </Link>
            <span class="spacer" />
            {!location.startsWith('/admin') && (
                <>
                    <select
                        class="select navbar-team-select"
                        aria-label="Current team"
                        value={$currentTeamId.value ?? ''}
                        onChange={(event) => setCurrentTeamId((event.target as HTMLSelectElement).value || null)}
                    >
                        {teams.length === 0 ? <option value="">No teams</option> : null}
                        {teams.map((team) => (
                            <option key={team.id} value={team.id}>
                                {team.name}
                            </option>
                        ))}
                    </select>
                    <span class="navbar-divider" />
                </>
            )}
            <div class="navbar-user-menu" ref={menuRef}>
                <button
                    class="navbar-user"
                    onClick={() => setMenuOpen((open) => !open)}
                    aria-haspopup="menu"
                    aria-expanded={menuOpen}
                >
                    <span class="navbar-user-avatar">{initials}</span>
                    <span class="navbar-user-name">{displayName}</span>
                </button>
                {menuOpen && (
                    <div class="navbar-dropdown" role="menu">
                        {user?.role === 'admin' && (
                            <button
                                class="navbar-item"
                                role="menuitem"
                                onClick={() => {
                                    setMenuOpen(false);
                                    navigate('/admin/users');
                                }}
                            >
                                <ShieldIcon class="is-sm" />
                                Admin
                            </button>
                        )}
                        <button
                            class="navbar-item"
                            role="menuitem"
                            onClick={() => {
                                setMenuOpen(false);
                                navigate('/settings');
                            }}
                        >
                            <CogIcon class="is-sm" />
                            Settings
                        </button>
                        <button
                            class="navbar-item"
                            role="menuitem"
                            onClick={async () => {
                                setMenuOpen(false);
                                await logout();
                            }}
                        >
                            <LogoutIcon class="is-sm" />
                            Logout
                        </button>
                    </div>
                )}
            </div>
        </nav>
    );
}
