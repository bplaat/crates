/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import { useEffect, useState } from 'preact/hooks';
import { type Session } from '../../../src-gen/api.ts';
import { Card } from '../../components/card.tsx';
import { ConfirmDialog } from '../../components/dialog.tsx';
import { SettingsLayout } from '../../components/settings-layout.tsx';
import { $currentSessionId } from '../../services/auth.service.ts';
import { formatDate, t } from '../../services/i18n.service.ts';
import { useInfiniteScroll } from '../../hooks/use-infinite-scroll.ts';
import { listSessions, revokeSession } from '../../services/sessions.service.ts';
import { HistoryIcon, LaptopIcon } from '../../components/icons.tsx';

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
    const { items: sessions, loading, hasMore, sentinelRef, setItems: setSessions } = useInfiniteScroll(listSessions);
    const [confirmRevokeId, setConfirmRevokeId] = useState<string | null>(null);
    const currentSessionId = $currentSessionId.value;

    useEffect(() => {
        document.title = `PlaatNotes - ${t('page.sessions')}`;
    }, []);

    function handleRevoke(id: string) {
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

                    {loading && sessions.length === 0 && (
                        <p class="text-center text-gray-400 dark:text-gray-500 mt-16">
                            {t('settings.sessions.loading')}
                        </p>
                    )}

                    {!loading && sessions.length === 0 && (
                        <p class="text-center text-gray-400 dark:text-gray-500 mt-16">{t('settings.sessions.empty')}</p>
                    )}

                    {sessions.length > 0 && (
                        <div class="flex flex-col gap-3">
                            {sessions.map((session) => {
                                const isCurrent = session.id === currentSessionId;
                                return (
                                    <Card key={session.id} class="px-5 py-4 flex items-start gap-4">
                                        <div class="mt-0.5 text-gray-400 dark:text-gray-500 shrink-0">
                                            <LaptopIcon class="w-6 h-6" />
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
                                                <HistoryIcon class="w-3.5 h-3.5" />
                                                {t('settings.sessions.revoke')}
                                            </button>
                                        )}
                                    </Card>
                                );
                            })}
                        </div>
                    )}

                    {hasMore && <div ref={sentinelRef} class="h-1" />}
                    {loading && sessions.length > 0 && (
                        <p class="text-center text-gray-400 dark:text-gray-500 py-4">
                            {t('settings.sessions.loading')}
                        </p>
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
