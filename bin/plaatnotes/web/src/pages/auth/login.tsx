/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import { route } from 'preact-router';
import { useEffect, useState } from 'preact/hooks';
import { login } from '../../services/auth.service.ts';

export function AuthLogin() {
    const [email, setEmail] = useState('');
    const [password, setPassword] = useState('');
    const [error, setError] = useState(false);
    const [loading, setLoading] = useState(false);

    useEffect(() => {
        document.title = 'PlaatNotes - Login';
    }, []);

    async function handleSubmit(event: SubmitEvent) {
        event.preventDefault();
        setError(false);
        setLoading(true);
        const success = await login(email, password);
        setLoading(false);
        if (success) {
            route('/');
        } else {
            setError(true);
        }
    }

    return (
        <div class="min-h-screen bg-gray-50 flex flex-col items-center justify-center p-4">
            <div class="mb-8 flex flex-col items-center gap-2">
                <img src="/assets/icon.svg" class="w-16 h-16" alt="" />

                <h1 class="text-3xl font-medium text-gray-700">PlaatNotes</h1>
                <p class="text-gray-500 text-sm">Sign in to your account</p>
            </div>

            <div class="bg-white rounded-2xl shadow-sm border border-gray-200 w-full max-w-sm p-8">
                {error && (
                    <div class="mb-4 p-3 bg-red-50 border border-red-200 rounded-lg text-red-600 text-sm">
                        Invalid email or password.
                    </div>
                )}

                <form onSubmit={handleSubmit} class="flex flex-col gap-4">
                    <div class="flex flex-col gap-1">
                        <label class="text-sm font-medium text-gray-700">Email</label>
                        <input
                            class="w-full px-3 py-2 border border-gray-300 rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-yellow-400 focus:border-transparent"
                            type="email"
                            required
                            placeholder="you@example.com"
                            value={email}
                            onInput={(e) => setEmail((e.target as HTMLInputElement).value)}
                        />
                    </div>

                    <div class="flex flex-col gap-1">
                        <label class="text-sm font-medium text-gray-700">Password</label>
                        <input
                            class="w-full px-3 py-2 border border-gray-300 rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-yellow-400 focus:border-transparent"
                            type="password"
                            required
                            placeholder="••••••••"
                            value={password}
                            onInput={(e) => setPassword((e.target as HTMLInputElement).value)}
                        />
                    </div>

                    <button
                        type="submit"
                        disabled={loading}
                        class="w-full mt-2 py-2 px-4 bg-yellow-400 hover:bg-yellow-500 disabled:opacity-60 text-white font-medium rounded-lg transition-colors cursor-pointer"
                    >
                        {loading ? 'Signing in…' : 'Sign in'}
                    </button>
                </form>
            </div>

            <p class="mt-8 text-xs text-gray-400">
                Made by{' '}
                <a
                    href="https://bplaat.nl"
                    target="_blank"
                    rel="noopener noreferrer"
                    class="underline hover:text-gray-600"
                >
                    Bastiaan van der Plaat
                </a>
            </p>
        </div>
    );
}
