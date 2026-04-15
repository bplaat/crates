/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import { useEffect, useRef, useState } from 'preact/hooks';
import { Button } from '../../components/button.tsx';
import { Card } from '../../components/card.tsx';
import { FormActions, FormField, FormMessage } from '../../components/form.tsx';
import { FormInput } from '../../components/input.tsx';
import { SettingsLayout } from '../../components/settings-layout.tsx';
import { authFetch } from '../../services/auth.service.ts';
import { t } from '../../services/i18n.service.ts';
import { CloudUploadIcon } from '../../components/icons.tsx';
import { type ImportGoogleKeepResponse } from '../../../src-gen/api.ts';

export function SettingsImports() {
    useEffect(() => {
        document.title = `PlaatNotes - ${t('settings.imports')}`;
    }, []);

    const [file, setFile] = useState<File | null>(null);
    const [loading, setLoading] = useState(false);
    const [importedCount, setImportedCount] = useState<number | null>(null);
    const [error, setError] = useState(false);
    const formRef = useRef<HTMLFormElement>(null);

    async function handleSubmit(e: SubmitEvent) {
        e.preventDefault();
        if (!file) return;
        setLoading(true);
        setImportedCount(null);
        setError(false);

        try {
            const arrayBuffer = await file.arrayBuffer();

            const res = await authFetch(`/api/imports/google-keep`, {
                method: 'POST',
                headers: { 'Content-Type': 'application/zip' },
                body: arrayBuffer,
            });
            if (!res.ok) {
                setError(true);
            } else {
                const { count }: ImportGoogleKeepResponse = await res.json();
                setImportedCount(count);
                setFile(null);
                formRef.current?.reset();
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

                <Card>
                    <h2 class="text-base font-semibold text-gray-700 dark:text-gray-200 mb-1">
                        {t('settings.imports.google_keep.heading')}
                    </h2>
                    <p class="text-sm text-gray-500 dark:text-gray-400 mb-5">
                        {t('settings.imports.google_keep.desc')}
                    </p>
                    <form ref={formRef} onSubmit={handleSubmit} class="flex flex-col gap-4">
                        <FormField id="keepFile" label={t('settings.imports.google_keep.label')}>
                            <FormInput
                                id="keepFile"
                                type="file"
                                accept=".zip"
                                required
                                onChange={(e) => setFile((e.target as HTMLInputElement).files?.[0] ?? null)}
                            />
                        </FormField>
                        <div class="flex flex-col gap-3 pt-1">
                            <div>
                                <FormMessage
                                    type="success"
                                    message={
                                        importedCount !== null &&
                                        t('settings.imports.google_keep.success', String(importedCount))
                                    }
                                />
                                <FormMessage type="error" message={error && t('settings.imports.google_keep.error')} />
                            </div>
                            <FormActions class="pt-0">
                                <Button type="submit" disabled={loading || !file}>
                                    <span class="flex items-center gap-1.5">
                                        <CloudUploadIcon class="w-4 h-4" />
                                        {loading
                                            ? t('settings.imports.google_keep.importing')
                                            : t('settings.imports.google_keep.submit')}
                                    </span>
                                </Button>
                            </FormActions>
                        </div>
                    </form>
                </Card>
            </div>
        </SettingsLayout>
    );
}
