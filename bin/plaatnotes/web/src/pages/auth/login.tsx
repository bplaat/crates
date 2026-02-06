/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import { useEffect, useState } from 'preact/hooks';
import { route } from '../../router.tsx';
import { AuthService } from '../../services/auth.service.ts';

export function Login() {
    const [email, setEmail] = useState<string>('');
    const [password, setPassword] = useState<string>('');
    const [error, setError] = useState<string>('');
    const [isLoading, setIsLoading] = useState<boolean>(false);

    useEffect(() => {
        document.title = 'PlaatNotes - Login';
    }, []);

    async function handleLogin(event: SubmitEvent) {
        event.preventDefault();
        setError('');
        setIsLoading(true);

        const success = await AuthService.getInstance().login(email, password);
        if (success) {
            route('/');
        } else {
            setError('Invalid email or password');
        }
        setIsLoading(false);
    }

    return (
        <div class="container">
            <h1 class="title">PlaatNotes</h1>

            <div class="box" style={{ maxWidth: '400px', margin: '2rem auto' }}>
                <h2 class="subtitle">Login</h2>

                {error && <div class="notification is-danger">{error}</div>}

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
                                required
                            />
                        </div>
                    </div>

                    <div class="field">
                        <label class="label">Password</label>
                        <div class="control">
                            <input
                                class="input"
                                type="password"
                                placeholder="Password"
                                value={password}
                                onInput={(e) => setPassword((e.target as HTMLInputElement).value)}
                                required
                            />
                        </div>
                    </div>

                    <div class="field is-grouped">
                        <div class="control">
                            <button class="button is-link" type="submit" disabled={isLoading}>
                                {isLoading ? 'Logging in...' : 'Login'}
                            </button>
                        </div>
                    </div>
                </form>
            </div>

            <p style={{ textAlign: 'center' }}>
                Made by <a href="https://bplaat.nl">Bastiaan van der Plaat</a>
            </p>
        </div>
    );
}
