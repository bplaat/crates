/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import { useEffect, useState } from 'preact/hooks';
import { type Session } from '../../../src-gen/api.ts';
import { ConfirmDialog } from '../../components/dialog.tsx';
import { SettingsLayout } from '../../components/settings-layout.tsx';
import { $currentSessionId } from '../../services/auth.service.ts';
import { formatDate, t } from '../../services/i18n.service.ts';
import { listSessions, revokeSession } from '../../services/sessions.service.ts';

function clientLabel(session: Session): string {
    const { name, version, os } = session.client;
    if (name && version && os) return `${name} ${version} on ${os}`;
    if (name && version) return `${name} ${version}`;
    if (name && os) return `${name} on ${os}`;
    if (name) return name;
    return '—';
}

function locationLabel(session: Session): string {
    const { address, city, country } = session.ip;
    const place = [city, country].filter(Boolean).join(', ');
    if (address && place) return `${address} (${place})`;
    if (address) return address;
    if (place) return place;
    return '—';
}

export function SettingsSessions() {
    const [sessions, setSessions] = useState<Session[]>([]);
    const [loading, setLoading] = useState(true);
    const [confirmRevokeId, setConfirmRevokeId] = useState<string | null>(null);
    const currentSessionId = $currentSessionId.value;

    // @ts-ignore
    useEffect(async () => {
        document.title = `PlaatNotes - ${t('page.sessions')}`;
        const data = await listSessions();
        setSessions(data);
        setLoading(false);
    }, []);

    async function handleRevoke(id: string) {
        setConfirmRevokeId(id);
    }

    async function doRevoke() {
        if (!confirmRevokeId) return;
        const ok = await revokeSession(confirmRevokeId);
        if (ok) setSessions((ss) => ss.filter((s) => s.id !== confirmRevokeId));
        setConfirmRevokeId(null);
    }

    return (
        <>
            <SettingsLayout>
                <div class="max-w-2xl mx-auto px-4 py-8">
                    <h1 class="text-xl font-medium text-gray-700 dark:text-gray-200 mb-6">
                        {t('settings.sessions.heading')}
                    </h1>

                    {loading && (
                        <p class="text-center text-gray-400 dark:text-gray-500 mt-16">
                            {t('settings.sessions.loading')}
                        </p>
                    )}

                    {!loading && sessions.length === 0 && (
                        <p class="text-center text-gray-400 dark:text-gray-500 mt-16">{t('settings.sessions.empty')}</p>
                    )}

                    {!loading && sessions.length > 0 && (
                        <div class="flex flex-col gap-3">
                            {sessions.map((session) => {
                                const isCurrent = session.id === currentSessionId;
                                return (
                                    <div
                                        key={session.id}
                                        class="bg-white dark:bg-zinc-800 rounded-2xl border border-gray-200 dark:border-zinc-700 shadow-sm px-5 py-4 flex items-start gap-4"
                                    >
                                        <div class="mt-0.5 text-gray-400 dark:text-gray-500 shrink-0">
                                            <svg class="w-6 h-6" viewBox="0 0 24 24" fill="currentColor">
                                                <path d="M4 6h18V4H4c-1.1 0-2 .9-2 2v11H0v3h14v-3H4V6zm19 2h-6c-.55 0-1 .45-1 1v10c0 .55.45 1 1 1h6c.55 0 1-.45 1-1V9c0-.55-.45-1-1-1zm-1 9h-4v-7h4v7z" />
                                            </svg>
                                        </div>

                                        <div class="flex-1 min-w-0">
                                            <div class="flex items-center gap-2 flex-wrap">
                                                <p class="text-sm font-medium text-gray-800 dark:text-gray-100 truncate">
                                                    {clientLabel(session)}
                                                </p>
                                                {isCurrent && (
                                                    <span class="inline-flex items-center px-2 py-0.5 rounded-full text-xs font-medium bg-yellow-100 dark:bg-yellow-900/40 text-yellow-700 dark:text-yellow-400">
                                                        {t('settings.sessions.current')}
                                                    </span>
                                                )}
                                            </div>
                                            <p class="text-xs text-gray-500 dark:text-gray-400 mt-0.5">
                                                {locationLabel(session)}
                                            </p>
                                            <p class="text-xs text-gray-400 dark:text-gray-500 mt-1">
                                                {t('settings.sessions.created', formatDate(session.createdAt))}
                                                {' · '}
                                                {t('settings.sessions.expires', formatDate(session.expiresAt))}
                                            </p>
                                        </div>

                                        {!isCurrent && (
                                            <button
                                                onClick={() => handleRevoke(session.id)}
                                                class="shrink-0 inline-flex items-center gap-1.5 text-xs px-3 py-1.5 rounded-lg border border-gray-200 dark:border-zinc-600 text-gray-500 dark:text-gray-400 hover:border-red-300 dark:hover:border-red-700 hover:text-red-500 dark:hover:text-red-400 transition-colors cursor-pointer"
                                            >
                                                <svg class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="currentColor">
                                                    <path d="M13 3a9 9 0 0 0-9 9H1l3.89 3.89.07.14L9 12H6c0-3.87 3.13-7 7-7s7 3.13 7 7-3.13 7-7 7c-1.93 0-3.68-.79-4.94-2.06l-1.42 1.42A8.954 8.954 0 0 0 13 21a9 9 0 0 0 0-18zm-1 5v5l4.28 2.54.72-1.21-3.5-2.08V8H12z" />
                                                </svg>
                                                {t('settings.sessions.revoke')}
                                            </button>
                                        )}
                                    </div>
                                );
                            })}
                        </div>
                    )}
                </div>
            </SettingsLayout>

            {confirmRevokeId && (
                <ConfirmDialog
                    title={t('settings.sessions.revoke')}
                    message={t('settings.sessions.confirm_revoke')}
                    confirmLabel={t('settings.sessions.revoke')}
                    onConfirm={doRevoke}
                    onClose={() => setConfirmRevokeId(null)}
                />
            )}
        </>
    );
}
