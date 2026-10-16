/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import {
    Button,
    Card,
    CardTitle,
    CloudUploadIcon,
    Form,
    FormActions,
    FormField,
    FormFooter,
    FormInput,
    FormMessage,
    IconText,
    Page,
    PageTitle,
} from 'plaatui';
import { useEffect, useRef, useState } from 'preact/hooks';
import { type ImportGoogleKeepResponse } from '../../../src-gen/api.ts';
import { SettingsLayout } from '../../components/settings-layout.tsx';
import { authFetch } from '../../services/auth.service.ts';
import { t } from '../../services/i18n.service.ts';

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
            <Page size="narrow">
                <PageTitle>{t('settings.imports.heading')}</PageTitle>

                <Card>
                    <CardTitle tight>{t('settings.imports.google_keep.heading')}</CardTitle>
                    <p class="card-desc">{t('settings.imports.google_keep.desc')}</p>
                    <Form ref={formRef} onSubmit={handleSubmit}>
                        <FormField id="keepFile" label={t('settings.imports.google_keep.label')}>
                            <FormInput
                                id="keepFile"
                                type="file"
                                accept=".zip"
                                required
                                onChange={(e) => setFile((e.target as HTMLInputElement).files?.[0] ?? null)}
                            />
                        </FormField>
                        <FormFooter>
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
                            <FormActions flush>
                                <Button type="submit" disabled={loading || !file}>
                                    <IconText>
                                        <CloudUploadIcon class="is-sm" />
                                        {loading
                                            ? t('settings.imports.google_keep.importing')
                                            : t('settings.imports.google_keep.submit')}
                                    </IconText>
                                </Button>
                            </FormActions>
                        </FormFooter>
                    </Form>
                </Card>
            </Page>
        </SettingsLayout>
    );
}
