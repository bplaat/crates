/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import { Button, Card, CardTitle, Form, FormActions, FormField, FormInput, FormMessage, LoginIcon } from 'plaatui';
import { useEffect, useState } from 'preact/hooks';
import { login } from '../../services/auth.service.ts';

export function Login() {
    const [email, setEmail] = useState('');
    const [password, setPassword] = useState('');
    const [loading, setLoading] = useState(false);
    const [error, setError] = useState('');
    useEffect(() => {
        document.title = 'Plaat Deploy - Log in';
    }, []);
    async function submit(event: SubmitEvent) {
        event.preventDefault();
        setLoading(true);
        setError('');
        const result = await login(email, password);
        setLoading(false);
        if (result !== 'success')
            setError(result === 'rate_limited' ? 'Too many attempts. Try again later.' : 'Invalid email or password.');
    }
    return (
        <main class="login-shell">
            <Card class="login-card">
                <div class="login-brand">
                    <span class="deploy-mark is-large" aria-hidden="true">
                        PD
                    </span>
                    <div>
                        <h1>Plaat Deploy</h1>
                        <p>Ship from GitHub to your own server.</p>
                    </div>
                </div>
                <CardTitle>Log in</CardTitle>
                <Form onSubmit={submit}>
                    <FormField id="email" label="Email">
                        <FormInput
                            id="email"
                            type="email"
                            required
                            autoFocus
                            value={email}
                            onInput={(e) => setEmail(e.currentTarget.value)}
                        />
                    </FormField>
                    <FormField id="password" label="Password">
                        <FormInput
                            id="password"
                            type="password"
                            required
                            value={password}
                            onInput={(e) => setPassword(e.currentTarget.value)}
                        />
                    </FormField>
                    <FormMessage type="error" message={error} />
                    <FormActions>
                        <Button type="submit" disabled={loading}>
                            <LoginIcon class="is-sm" />
                            {loading ? 'Logging in...' : 'Log in'}
                        </Button>
                    </FormActions>
                </Form>
            </Card>
        </main>
    );
}
