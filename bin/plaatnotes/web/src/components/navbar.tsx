/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import { Link, route } from 'preact-router';
import { useEffect, useRef, useState } from 'preact/hooks';
import { $authUser, logout } from '../services/auth.service.ts';
import { t } from '../services/i18n.service.ts';

export function Navbar() {
    const user = $authUser.value;
    const [dropdownOpen, setDropdownOpen] = useState(false);
    const dropdownRef = useRef<HTMLDivElement>(null);

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
        route('/auth/login');
    }

    return (
        <header class="bg-white dark:bg-zinc-800 shadow-sm sticky top-0 z-10">
            <div class="px-4 h-16 flex items-center gap-4">
                <Link
                    href="/"
                    class="flex items-center gap-2 text-gray-700 dark:text-gray-200 hover:text-gray-900 dark:hover:text-white no-underline"
                >
                    <img src="/assets/icon.svg" class="w-8 h-8" alt="" />
                    <span class="text-xl font-medium">PlaatNotes</span>
                </Link>

                <div class="flex-1" />

                {user && (
                    <div class="relative" ref={dropdownRef}>
                        <button
                            onClick={() => setDropdownOpen(!dropdownOpen)}
                            class="flex items-center gap-3 rounded-lg px-2 py-1.5 hover:bg-gray-100 dark:hover:bg-zinc-700 transition-colors cursor-pointer"
                        >
                            <div class="w-8 h-8 rounded-full bg-yellow-400 dark:bg-yellow-900/40 text-white dark:text-yellow-400 font-semibold text-sm flex items-center justify-center select-none">
                                {user.firstName[0]}
                                {user.lastName[0]}
                            </div>
                            <span class="text-sm text-gray-600 dark:text-gray-300 hidden sm:block">
                                {user.firstName} {user.lastName}
                            </span>
                        </button>

                        {dropdownOpen && (
                            <div class="absolute right-0 mt-1 w-44 bg-white dark:bg-zinc-800 border border-gray-200 dark:border-zinc-700 rounded-xl shadow-lg overflow-hidden z-20">
                                {user.role === 'admin' && (
                                    <>
                                        <button
                                            onClick={() => {
                                                setDropdownOpen(false);
                                                route('/admin/users');
                                            }}
                                            class="w-full flex items-center gap-2.5 px-4 py-2.5 text-sm text-gray-700 dark:text-gray-200 hover:bg-gray-50 dark:hover:bg-zinc-700 transition-colors cursor-pointer"
                                        >
                                            <svg
                                                class="w-4 h-4 text-gray-400 dark:text-gray-500 shrink-0"
                                                viewBox="0 0 24 24"
                                                fill="currentColor"
                                            >
                                                <path d="M12 1L3 5v6c0 5.55 3.84 10.74 9 12 5.16-1.26 9-6.45 9-12V5l-9-4zm0 4l4 1.78V11c0 3.35-2.32 6.48-4 7.44-1.68-.96-4-4.09-4-7.44V6.78L12 5z" />
                                            </svg>
                                            {t('nav.admin')}
                                        </button>
                                        <div class="border-t border-gray-100 dark:border-zinc-700" />
                                    </>
                                )}
                                <button
                                    onClick={() => {
                                        setDropdownOpen(false);
                                        route('/settings');
                                    }}
                                    class="w-full flex items-center gap-2.5 px-4 py-2.5 text-sm text-gray-700 dark:text-gray-200 hover:bg-gray-50 dark:hover:bg-zinc-700 transition-colors cursor-pointer"
                                >
                                    <svg
                                        class="w-4 h-4 text-gray-400 dark:text-gray-500 shrink-0"
                                        viewBox="0 0 24 24"
                                        fill="currentColor"
                                    >
                                        <path d="M19.14 12.94c.04-.3.06-.61.06-.94 0-.32-.02-.64-.07-.94l2.03-1.58c.18-.14.23-.41.12-.61l-1.92-3.32c-.12-.22-.37-.29-.59-.22l-2.39.96c-.5-.38-1.03-.7-1.62-.94l-.36-2.54c-.04-.24-.24-.41-.48-.41h-3.84c-.24 0-.43.17-.47.41l-.36 2.54c-.59.24-1.13.57-1.62.94l-2.39-.96c-.22-.08-.47 0-.59.22L2.74 8.87c-.12.21-.08.47.12.61l2.03 1.58c-.05.3-.09.63-.09.94s.02.64.07.94l-2.03 1.58c-.18.14-.23.41-.12.61l1.92 3.32c.12.22.37.29.59.22l2.39-.96c.5.38 1.03.7 1.62.94l.36 2.54c.05.24.24.41.48.41h3.84c.24 0 .44-.17.47-.41l.36-2.54c.59-.24 1.13-.56 1.62-.94l2.39.96c.22.08.47 0 .59-.22l1.92-3.32c.12-.22.07-.47-.12-.61l-2.01-1.58zM12 15.6c-1.98 0-3.6-1.62-3.6-3.6s1.62-3.6 3.6-3.6 3.6 1.62 3.6 3.6-1.62 3.6-3.6 3.6z" />
                                    </svg>
                                    {t('nav.settings')}
                                </button>
                                <div class="border-t border-gray-100 dark:border-zinc-700" />
                                <button
                                    onClick={handleLogout}
                                    class="w-full flex items-center gap-2.5 px-4 py-2.5 text-sm text-gray-700 dark:text-gray-200 hover:bg-gray-50 dark:hover:bg-zinc-700 transition-colors cursor-pointer"
                                >
                                    <svg
                                        class="w-4 h-4 text-gray-400 dark:text-gray-500 shrink-0"
                                        viewBox="0 0 24 24"
                                        fill="currentColor"
                                    >
                                        <path d="M17 7l-1.41 1.41L18.17 11H8v2h10.17l-2.58 2.58L17 17l5-5zM4 5h8V3H4c-1.1 0-2 .9-2 2v14c0 1.1.9 2 2 2h8v-2H4V5z" />
                                    </svg>
                                    {t('nav.logout')}
                                </button>
                            </div>
                        )}
                    </div>
                )}
            </div>
        </header>
    );
}
