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
        <header class="bg-white dark:bg-zinc-800 shadow-sm sticky top-0 z-10">
            <div class="px-4 h-16 relative flex items-center">
                <Link
                    href="/"
                    class="flex items-center gap-2 text-gray-700 dark:text-gray-200 hover:text-gray-900 dark:hover:text-white no-underline shrink-0"
                >
                    <img src="/assets/icon.svg" class="w-8 h-8" alt="" />
                    <span class="text-xl font-medium hidden sm:block">PlaatNotes</span>
                </Link>

                {showSearch && (
                    <div class="absolute inset-0 flex items-center justify-center pointer-events-none px-4">
                        <div class="w-full max-w-md pointer-events-auto">
                            <SearchInput
                                value={$searchQuery.value}
                                onInput={(v) => ($searchQuery.value = v)}
                                onClear={() => ($searchQuery.value = '')}
                                placeholder={t('nav.search')}
                            />
                        </div>
                    </div>
                )}

                <div class="flex-1" />

                {user && (
                    <div class="relative" ref={dropdownRef}>
                        <button
                            onClick={() => setDropdownOpen(!dropdownOpen)}
                            class="flex items-center gap-3 rounded-lg px-2 py-1.5 hover:bg-gray-100 dark:hover:bg-zinc-700 transition-colors cursor-pointer"
                        >
                            <div class="w-8 h-8 rounded-full bg-yellow-400 dark:bg-yellow-900/40 text-white dark:text-yellow-400 font-semibold text-xs flex items-center justify-center select-none">
                                {user.firstName[0].toUpperCase()}
                                {lastNameInitial(user.lastName)}
                            </div>
                            <span class="text-sm text-gray-600 dark:text-gray-300 hidden sm:block">
                                {user.firstName} {user.lastName}
                            </span>
                        </button>

                        {dropdownOpen && (
                            <div class="absolute right-0 mt-1 w-44 bg-white dark:bg-zinc-800 border border-gray-200 dark:border-zinc-700 rounded-xl shadow-lg overflow-hidden z-20">
                                {user.role === 'admin' && (
                                    <>
                                        <DropdownItem
                                            onClick={() => {
                                                setDropdownOpen(false);
                                                navigate('/admin/users');
                                            }}
                                        >
                                            <ShieldIcon class="w-4 h-4 text-gray-400 dark:text-gray-500 shrink-0" />
                                            {t('nav.admin')}
                                        </DropdownItem>
                                        <div class="border-t border-gray-100 dark:border-zinc-700" />
                                    </>
                                )}
                                <DropdownItem
                                    onClick={() => {
                                        setDropdownOpen(false);
                                        navigate('/settings');
                                    }}
                                >
                                    <CogIcon class="w-4 h-4 text-gray-400 dark:text-gray-500 shrink-0" />
                                    {t('nav.settings')}
                                </DropdownItem>
                                <div class="border-t border-gray-100 dark:border-zinc-700" />
                                <DropdownItem onClick={handleLogout}>
                                    <LogoutIcon class="w-4 h-4 text-gray-400 dark:text-gray-500 shrink-0" />
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
        <button
            onClick={onClick}
            class="w-full flex items-center gap-2.5 px-4 py-2.5 text-sm text-gray-700 dark:text-gray-200 hover:bg-gray-50 dark:hover:bg-zinc-700 transition-colors cursor-pointer"
        >
            {children}
        </button>
    );
}
