/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import { useEffect, useState } from 'preact/hooks';
import { $loading, login } from '../auth.ts';
import { route } from '../router.tsx';

export function Login() {
    const [email, setEmail] = useState('');
    const [password, setPassword] = useState('');
    const [error, setError] = useState('');

    useEffect(() => {
        document.title = 'PlaatNotes - Login';
    }, []);

    async function handleLogin(e: SubmitEvent) {
        e.preventDefault();
        setError('');

        if (!email || !password) {
            setError('Email and password are required');
            return;
        }

        const success = await login(email, password);
        if (success) {
            route('/');
        } else {
            setError('Invalid email or password');
        }
    }

    return (
        <div class="section is-fullheight has-background-light">
            <div class="container">
                <div class="columns is-centered">
                    <div class="column is-5-tablet is-4-desktop">
                        <div class="box">
                            <h1 class="title has-text-centered">PlaatNotes</h1>

                            {error && (
                                <div class="notification is-danger is-light">
                                    <button class="delete" onClick={() => setError('')} />
                                    {error}
                                </div>
                            )}

                            <form onSubmit={handleLogin}>
                                <div class="field">
                                    <label class="label">Email</label>
                                    <div class="control">
                                        <input
                                            class="input"
                                            type="email"
                                            placeholder="your@email.com"
                                            value={email}
                                            onInput={(e) => setEmail((e.target as HTMLInputElement).value)}
                                        />
                                    </div>
                                </div>

                                <div class="field">
                                    <label class="label">Password</label>
                                    <div class="control">
                                        <input
                                            class="input"
                                            type="password"
                                            placeholder="••••••••"
                                            value={password}
                                            onInput={(e) => setPassword((e.target as HTMLInputElement).value)}
                                        />
                                    </div>
                                </div>

                                <div class="field">
                                    <div class="control">
                                        <button
                                            class={`button is-link is-fullwidth ${$loading.value ? 'is-loading' : ''}`}
                                            type="submit"
                                            disabled={$loading.value}
                                        >
                                            Login
                                        </button>
                                    </div>
                                </div>
                            </form>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    );
}
