/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import { useLocation } from 'wouter-preact';
import { useEffect, useState } from 'preact/hooks';
import { Button } from '../../components/button.tsx';
import { FormActions, FormField } from '../../components/form.tsx';
import { FormInput } from '../../components/input.tsx';
import { login } from '../../services/auth.service.ts';
import { t } from '../../services/i18n.service.ts';
import { LoginIcon } from '../../components/icons.tsx';

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
        <div class="min-h-screen bg-gray-50 dark:bg-zinc-900 flex flex-col items-center justify-center p-4">
            <div class="mb-8 flex flex-col items-center gap-2">
                <img src="/assets/icon.svg" class="w-16 h-16" alt="" />

                <h1 class="text-3xl font-medium text-gray-700 dark:text-gray-200">PlaatNotes</h1>
                <p class="text-gray-500 dark:text-gray-400 text-sm">{t('login.tagline')}</p>
            </div>

            <div class="bg-white dark:bg-zinc-800 rounded-2xl shadow-sm border border-gray-200 dark:border-zinc-700 w-full max-w-sm p-8">
                {error && (
                    <div class="mb-4 p-3 bg-red-50 dark:bg-red-900/30 border border-red-200 dark:border-red-700 rounded-lg text-red-600 dark:text-red-400 text-sm">
                        {t(error === 'rate_limited' ? 'login.error_rate_limited' : 'login.error')}
                    </div>
                )}

                <form onSubmit={handleSubmit} class="flex flex-col gap-4">
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
                            <span class="flex items-center gap-1.5">
                                <LoginIcon class="w-4 h-4" />
                                {loading ? t('login.submitting') : t('login.submit')}
                            </span>
                        </Button>
                    </FormActions>
                </form>
            </div>

            <p class="mt-8 text-xs text-gray-400 dark:text-gray-500">
                {t('login.made_by')}{' '}
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
