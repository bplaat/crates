/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import { Button, Form, FormActions, FormField, FormInput, IconText, LoginIcon } from 'plaatui';
import { useEffect, useState } from 'preact/hooks';
import { useLocation } from 'wouter-preact';
import { login } from '../../services/auth.service.ts';
import { t } from '../../services/i18n.service.ts';
import './login.css';

export function AuthLogin() {
    const [email, setEmail] = useState('');
    const [password, setPassword] = useState('');
    const [error, setError] = useState<'error' | 'rate_limited' | null>(null);
    const [loading, setLoading] = useState(false);
    const [, navigate] = useLocation();

    useEffect(() => {
        document.title = `PlaatNotes - ${t('page.login')}`;
    }, []);

    async function handleSubmit(event: SubmitEvent) {
        event.preventDefault();
        setError(null);
        setLoading(true);
        const result = await login(email, password);
        setLoading(false);
        if (result === 'success') {
            navigate('/');
        } else {
            setError(result);
        }
    }

    return (
        <div class="login">
            <div class="login-header">
                <img src="/assets/icon.svg" alt="" />

                <h1 class="login-title">PlaatNotes</h1>
                <p class="login-tagline">{t('login.tagline')}</p>
            </div>

            <div class="login-card">
                {error && (
                    <div class="login-error">
                        {t(error === 'rate_limited' ? 'login.error_rate_limited' : 'login.error')}
                    </div>
                )}

                <Form onSubmit={handleSubmit}>
                    <FormField id="email" label={t('login.email')}>
                        <FormInput
                            id="email"
                            type="email"
                            required
                            placeholder={t('login.email_placeholder')}
                            value={email}
                            onInput={(e) => setEmail((e.target as HTMLInputElement).value)}
                        />
                    </FormField>

                    <FormField id="password" label={t('login.password')}>
                        <FormInput
                            id="password"
                            type="password"
                            required
                            placeholder="••••••••"
                            value={password}
                            onInput={(e) => setPassword((e.target as HTMLInputElement).value)}
                        />
                    </FormField>

                    <FormActions>
                        <Button type="submit" disabled={loading}>
                            <IconText>
                                <LoginIcon class="is-sm" />
                                {loading ? t('login.submitting') : t('login.submit')}
                            </IconText>
                        </Button>
                    </FormActions>
                </Form>
            </div>

            <p class="login-footer">
                {t('login.made_by')}{' '}
                <a href="https://bplaat.nl" target="_blank" rel="noopener noreferrer">
                    Bastiaan van der Plaat
                </a>
            </p>
        </div>
    );
}
