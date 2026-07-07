/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import { Link, useLocation } from 'wouter-preact';
import { useEffect, useRef, useState } from 'preact/hooks';
import { type ComponentChildren } from 'preact';
import { $authUser, logout } from '../services/auth.service.ts';
import { $searchQuery } from '../services/notes.service.ts';
import { t } from '../services/i18n.service.ts';
import { SearchInput } from './input.tsx';
import { CogIcon, LogoutIcon, ShieldIcon } from './icons.tsx';
import { lastNameInitial } from '../utils.ts';

export function Navbar({ showSearch = false }: { showSearch?: boolean }) {
    const user = $authUser.value;
    const [dropdownOpen, setDropdownOpen] = useState(false);
    const dropdownRef = useRef<HTMLDivElement>(null);
    const [, navigate] = useLocation();

    useEffect(() => {
        if (!showSearch) $searchQuery.value = '';
    }, [showSearch]);

    useEffect(() => {
        function handleClickOutside(e: MouseEvent) {
            if (dropdownRef.current && !dropdownRef.current.contains(e.target as Node)) {
                setDropdownOpen(false);
            }
        }
        if (dropdownOpen) {
            document.addEventListener('mousedown', handleClickOutside);
            return () => document.removeEventListener('mousedown', handleClickOutside);
        }
    }, [dropdownOpen]);

    async function handleLogout() {
        setDropdownOpen(false);
        await logout();
        navigate('/auth/login');
    }

    return (
        <header class="navbar">
            <div class="navbar-container">
                <Link href="/" class="navbar-brand">
                    <img src="/assets/icon.svg" alt="" />
                    <span class="navbar-brand-name">PlaatNotes</span>
                </Link>

                {showSearch && (
                    <div class="navbar-search">
                        <div class="navbar-search-inner">
                            <SearchInput
                                value={$searchQuery.value}
                                onInput={(v) => ($searchQuery.value = v)}
                                onClear={() => ($searchQuery.value = '')}
                                placeholder={t('nav.search')}
                            />
                        </div>
                    </div>
                )}

                <div class="spacer" />

                {user && (
                    <div class="navbar-menu-wrapper" ref={dropdownRef}>
                        <button onClick={() => setDropdownOpen(!dropdownOpen)} class="navbar-user">
                            <div class="avatar">
                                {user.firstName[0].toUpperCase()}
                                {lastNameInitial(user.lastName)}
                            </div>
                            <span class="navbar-user-name">
                                {user.firstName} {user.lastName}
                            </span>
                        </button>

                        {dropdownOpen && (
                            <div class="dropdown-menu">
                                {user.role === 'admin' && (
                                    <>
                                        <DropdownItem
                                            onClick={() => {
                                                setDropdownOpen(false);
                                                navigate('/admin/users');
                                            }}
                                        >
                                            <ShieldIcon class="is-sm" />
                                            {t('nav.admin')}
                                        </DropdownItem>
                                        <div class="dropdown-divider" />
                                    </>
                                )}
                                <DropdownItem
                                    onClick={() => {
                                        setDropdownOpen(false);
                                        navigate('/settings');
                                    }}
                                >
                                    <CogIcon class="is-sm" />
                                    {t('nav.settings')}
                                </DropdownItem>
                                <div class="dropdown-divider" />
                                <DropdownItem onClick={handleLogout}>
                                    <LogoutIcon class="is-sm" />
                                    {t('nav.logout')}
                                </DropdownItem>
                            </div>
                        )}
                    </div>
                )}
            </div>
        </header>
    );
}

function DropdownItem({ onClick, children }: { onClick: () => void; children: ComponentChildren }) {
    return (
        <button onClick={onClick} class="dropdown-item">
            {children}
        </button>
    );
}
