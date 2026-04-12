/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import { useState } from 'preact/hooks';
import { login } from '../services/auth.ts';

export function LoginPage() {
    const [email, setEmail] = useState('');
    const [password, setPassword] = useState('');
    const [error, setError] = useState('');
    const [loading, setLoading] = useState(false);

    async function handleSubmit(e: Event) {
        e.preventDefault();
        setLoading(true);
        setError('');
        const result = await login(email, password);
        setLoading(false);
        if (result === 'rate_limited') setError('Too many login attempts. Try again later.');
        else if (result === 'error') setError('Invalid email or password.');
    }

    return (
        <div class="login-wrap">
            <div class="login-box">
                <h1>PlaatDeploy</h1>
                {error && <div class="alert alert-error">{error}</div>}
                <form onSubmit={handleSubmit}>
                    <div class="form-group">
                        <label>Email</label>
                        <input
                            type="email"
                            value={email}
                            onInput={(e) => setEmail((e.target as HTMLInputElement).value)}
                            required
                        />
                    </div>
                    <div class="form-group">
                        <label>Password</label>
                        <input
                            type="password"
                            value={password}
                            onInput={(e) => setPassword((e.target as HTMLInputElement).value)}
                            required
                        />
                    </div>
                    <button class="btn btn-primary" style="width: 100%;" disabled={loading}>
                        {loading ? 'Logging in...' : 'Login'}
                    </button>
                </form>
            </div>
        </div>
    );
}
