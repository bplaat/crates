/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import { useState } from 'preact/hooks';
import { login } from '../services/auth.ts';
import { Button } from '../components/button.tsx';
import { FormField } from '../components/form.tsx';
import { FormInput } from '../components/input.tsx';
import { useDocumentTitle } from '../utils.ts';

export function LoginPage() {
    const [email, setEmail] = useState('');
    const [password, setPassword] = useState('');
    const [error, setError] = useState('');
    const [loading, setLoading] = useState(false);
    useDocumentTitle('Login');

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
                <div class="login-brand">
                    <h1>PlaatDeploy</h1>
                </div>
                {error && <div class="notification is-danger">{error}</div>}
                <form onSubmit={handleSubmit}>
                    <FormField id="login-email" label="Email">
                        <FormInput
                            id="login-email"
                            type="email"
                            value={email}
                            onInput={(e) => setEmail((e.target as HTMLInputElement).value)}
                            required
                        />
                    </FormField>
                    <FormField id="login-password" label="Password">
                        <FormInput
                            id="login-password"
                            type="password"
                            value={password}
                            onInput={(e) => setPassword((e.target as HTMLInputElement).value)}
                            required
                        />
                    </FormField>
                    <Button type="submit" class="is-full-width" disabled={loading}>
                        {loading ? 'Logging in…' : 'Login'}
                    </Button>
                </form>
            </div>
        </div>
    );
}
