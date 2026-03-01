/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import { useEffect, useState } from 'preact/hooks';
import { type Session } from '../../../src-gen/api.ts';
import { SettingsLayout } from '../../components/settings-layout.tsx';
import { $currentSessionId } from '../../services/auth.service.ts';
import { $language, t } from '../../services/i18n.service.ts';
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
    const currentSessionId = $currentSessionId.value;
    const locale = $language.value;
    const now = Date.now();

    // @ts-ignore
    useEffect(async () => {
        document.title = `PlaatNotes - ${t('page.sessions')}`;
        const data = await listSessions();
        setSessions(data.filter((s) => new Date(s.expiresAt).getTime() > now));
        setLoading(false);
    }, []);

    async function handleRevoke(id: string) {
        if (!confirm(t('sessions.confirm_revoke'))) return;
        const ok = await revokeSession(id);
        if (ok) setSessions((ss) => ss.filter((s) => s.id !== id));
    }

    return (
        <SettingsLayout>
            <div class="max-w-2xl mx-auto px-4 py-8">
                <h1 class="text-xl font-medium text-gray-700 dark:text-gray-200 mb-6">{t('sessions.heading')}</h1>

                {loading && <p class="text-center text-gray-400 dark:text-gray-500 mt-16">{t('sessions.loading')}</p>}

                {!loading && sessions.length === 0 && (
                    <p class="text-center text-gray-400 dark:text-gray-500 mt-16">{t('sessions.empty')}</p>
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
                                                    {t('sessions.current')}
                                                </span>
                                            )}
                                        </div>
                                        <p class="text-xs text-gray-500 dark:text-gray-400 mt-0.5">
                                            {locationLabel(session)}
                                        </p>
                                        <p class="text-xs text-gray-400 dark:text-gray-500 mt-1">
                                            {t('sessions.created', new Date(session.createdAt).toLocaleString(locale))}
                                            {' · '}
                                            {t('sessions.expires', new Date(session.expiresAt).toLocaleString(locale))}
                                        </p>
                                    </div>

                                    {!isCurrent && (
                                        <button
                                            onClick={() => handleRevoke(session.id)}
                                            class="shrink-0 text-xs px-3 py-1.5 rounded-lg border border-gray-200 dark:border-zinc-600 text-gray-500 dark:text-gray-400 hover:border-red-300 dark:hover:border-red-700 hover:text-red-500 dark:hover:text-red-400 transition-colors cursor-pointer"
                                        >
                                            {t('sessions.revoke')}
                                        </button>
                                    )}
                                </div>
                            );
                        })}
                    </div>
                )}
            </div>
        </SettingsLayout>
    );
}
