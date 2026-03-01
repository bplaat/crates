/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import { Link, route } from 'preact-router';
import { $authUser, logout } from '../services/auth.service.ts';
import { t } from '../services/i18n.service.ts';

export function Navbar() {
    const user = $authUser.value;

    async function handleLogout() {
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
                    <div class="flex items-center gap-3">
                        <div class="w-8 h-8 rounded-full bg-yellow-400 dark:bg-yellow-900/40 text-white dark:text-yellow-400 font-semibold text-sm flex items-center justify-center select-none">
                            {user.firstName[0]}
                            {user.lastName[0]}
                        </div>
                        <span class="text-sm text-gray-600 dark:text-gray-300 hidden sm:block">
                            {user.firstName} {user.lastName}
                        </span>
                        <button
                            onClick={handleLogout}
                            class="text-sm px-3 py-1.5 rounded-lg border border-gray-200 dark:border-zinc-600 text-gray-600 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-zinc-700 transition-colors cursor-pointer"
                        >
                            {t('nav.logout')}
                        </button>
                    </div>
                )}
            </div>
        </header>
    );
}
