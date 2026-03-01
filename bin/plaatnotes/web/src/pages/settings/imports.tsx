/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import { useEffect, useState } from 'preact/hooks';
import { Button, FormField, FormInput } from '../../components/form.tsx';
import { SettingsLayout } from '../../components/settings-layout.tsx';
import { API_URL } from '../../consts.ts';
import { authFetch } from '../../services/auth.service.ts';
import { t } from '../../services/i18n.service.ts';
import { type ImportGoogleKeepResponse } from '../../../src-gen/api.ts';

const CARD_CLASS = 'bg-white dark:bg-zinc-800 rounded-2xl border border-gray-200 dark:border-zinc-700 shadow-sm p-6';

export function SettingsImports() {
    useEffect(() => {
        document.title = `PlaatNotes - ${t('settings.imports')}`;
    }, []);

    const [file, setFile] = useState<File | null>(null);
    const [loading, setLoading] = useState(false);
    const [importedCount, setImportedCount] = useState<number | null>(null);
    const [error, setError] = useState(false);

    async function handleSubmit(e: SubmitEvent) {
        e.preventDefault();
        if (!file) return;
        setLoading(true);
        setImportedCount(null);
        setError(false);

        try {
            const arrayBuffer = await file.arrayBuffer();
            const bytes = new Uint8Array(arrayBuffer);
            // Encode binary to base64
            let binary = '';
            for (let i = 0; i < bytes.byteLength; i++) binary += String.fromCharCode(bytes[i]);
            const fileB64 = btoa(binary);

            const form = new URLSearchParams({ file: fileB64 });
            const res = await authFetch(`${API_URL}/imports/google-keep`, {
                method: 'POST',
                body: form,
            });
            if (!res.ok) {
                setError(true);
            } else {
                const { count }: ImportGoogleKeepResponse = await res.json();
                setImportedCount(count);
            }
        } catch {
            setError(true);
        }
        setLoading(false);
    }

    return (
        <SettingsLayout>
            <div class="max-w-2xl mx-auto px-4 py-8">
                <h1 class="text-xl font-medium text-gray-700 dark:text-gray-200 mb-6">
                    {t('settings.imports.heading')}
                </h1>

                <div class={CARD_CLASS}>
                    <h2 class="text-base font-semibold text-gray-700 dark:text-gray-200 mb-1">
                        {t('settings.imports.google_keep.heading')}
                    </h2>
                    <p class="text-sm text-gray-500 dark:text-gray-400 mb-5">
                        {t('settings.imports.google_keep.desc')}
                    </p>
                    <form onSubmit={handleSubmit} class="flex flex-col gap-4">
                        <FormField id="keepFile" label={t('settings.imports.google_keep.label')}>
                            <FormInput
                                id="keepFile"
                                type="file"
                                accept=".zip"
                                required
                                onChange={(e) => setFile((e.target as HTMLInputElement).files?.[0] ?? null)}
                            />
                        </FormField>
                        <div class="flex items-center justify-between pt-1">
                            <div>
                                {importedCount !== null && (
                                    <p class="text-sm text-green-600 dark:text-green-400">
                                        {t('settings.imports.google_keep.success').replace(
                                            '{0}',
                                            String(importedCount),
                                        )}
                                    </p>
                                )}
                                {error && (
                                    <p class="text-sm text-red-500 dark:text-red-400">
                                        {t('settings.imports.google_keep.error')}
                                    </p>
                                )}
                            </div>
                            <Button type="submit" disabled={loading || !file}>
                                {loading
                                    ? t('settings.imports.google_keep.importing')
                                    : t('settings.imports.google_keep.submit')}
                            </Button>
                        </div>
                    </form>
                </div>
            </div>
        </SettingsLayout>
    );
}
