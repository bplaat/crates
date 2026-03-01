/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import { Link, route } from 'preact-router';
import { $authUser, logout } from '../services/auth.service.ts';

export function Navbar() {
    const user = $authUser.value;

    async function handleLogout() {
        await logout();
        route('/auth/login');
    }

    return (
        <header class="bg-white shadow-sm sticky top-0 z-10">
            <div class="px-4 h-16 flex items-center gap-4">
                <Link href="/" class="flex items-center gap-2 text-gray-700 hover:text-gray-900 no-underline">
                    <img src="/assets/icon.svg" class="w-8 h-8" alt="" />
                    <span class="text-xl font-medium">PlaatNotes</span>
                </Link>

                <div class="flex-1" />

                {user && (
                    <div class="flex items-center gap-3">
                        <div class="w-8 h-8 rounded-full bg-yellow-400 text-white font-semibold text-sm flex items-center justify-center select-none">
                            {user.firstName[0]}
                            {user.lastName[0]}
                        </div>
                        <span class="text-sm text-gray-600 hidden sm:block">
                            {user.firstName} {user.lastName}
                        </span>
                        <button
                            onClick={handleLogout}
                            class="text-sm px-3 py-1.5 rounded-lg border border-gray-200 text-gray-600 hover:bg-gray-100 transition-colors cursor-pointer"
                        >
                            Logout
                        </button>
                    </div>
                )}
            </div>
        </header>
    );
}
