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
                <div class="page is-narrow">
                    <h1 class="page-title">{t('settings.sessions.heading')}</h1>

                    {loading && sessions.length === 0 && (
                        <p class="loading-text is-initial">{t('settings.sessions.loading')}</p>
                    )}

                    {!loading && sessions.length === 0 && (
                        <p class="loading-text is-initial">{t('settings.sessions.empty')}</p>
                    )}

                    {sessions.length > 0 && (
                        <div class="list">
                            {sessions.map((session) => {
                                const isCurrent = session.id === currentSessionId;
                                return (
                                    <Card key={session.id} class="session">
                                        <div class="session-icon">
                                            <LaptopIcon class="is-lg" />
                                        </div>

                                        <div class="session-body">
                                            <div class="session-head">
                                                <p class="session-name">{clientLabel(session)}</p>
                                                {isCurrent && (
                                                    <span class="badge is-accent">
                                                        {t('settings.sessions.current')}
                                                    </span>
                                                )}
                                            </div>
                                            <p class="session-location">{locationLabel(session)}</p>
                                            <p class="session-meta">
                                                {t('settings.sessions.created', formatDate(session.createdAt))}
                                                {' · '}
                                                {t('settings.sessions.expires', formatDate(session.expiresAt))}
                                            </p>
                                        </div>

                                        {!isCurrent && (
                                            <button onClick={() => handleRevoke(session.id)} class="session-revoke">
                                                <HistoryIcon class="is-xs" />
                                                {t('settings.sessions.revoke')}
                                            </button>
                                        )}
                                    </Card>
                                );
                            })}
                        </div>
                    )}

                    {hasMore && <div ref={sentinelRef} class="sentinel" />}
                    {loading && sessions.length > 0 && <p class="loading-text">{t('settings.sessions.loading')}</p>}
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
