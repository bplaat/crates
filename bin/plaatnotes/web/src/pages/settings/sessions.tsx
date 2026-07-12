/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import { useEffect, useState } from 'preact/hooks';
import '../../components/list.css';
import { type Session } from '../../../src-gen/api.ts';
import { Card } from 'plaatui';
import { ConfirmDialog } from 'plaatui';
import { SettingsLayout } from '../../components/settings-layout.tsx';
import { $currentSessionId } from '../../services/auth.service.ts';
import { formatDate, t } from '../../services/i18n.service.ts';
import { useInfiniteScroll } from '../../hooks/use-infinite-scroll.ts';
import { listSessions, revokeSession } from '../../services/sessions.service.ts';
import { Badge, Icon, LoadingText, SecondaryButton } from 'plaatui';
import './sessions.css';

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
                        <LoadingText initial>{t('settings.sessions.loading')}</LoadingText>
                    )}

                    {!loading && sessions.length === 0 && (
                        <LoadingText initial>{t('settings.sessions.empty')}</LoadingText>
                    )}

                    {sessions.length > 0 && (
                        <div class="list">
                            {sessions.map((session) => {
                                const isCurrent = session.id === currentSessionId;
                                return (
                                    <Card key={session.id} class="session" padded={false}>
                                        <div class="session-icon">
                                            <Icon type="laptop" class="is-lg" />
                                        </div>

                                        <div class="session-body">
                                            <div class="session-head">
                                                <p class="session-name">{clientLabel(session)}</p>
                                                {isCurrent && <Badge accent>{t('settings.sessions.current')}</Badge>}
                                            </div>
                                            <p class="session-location">{locationLabel(session)}</p>
                                            <p class="session-meta">
                                                {t('settings.sessions.created', formatDate(session.createdAt))}
                                                {' · '}
                                                {t('settings.sessions.expires', formatDate(session.expiresAt))}
                                            </p>
                                        </div>

                                        {!isCurrent && (
                                            <SecondaryButton onClick={() => handleRevoke(session.id)}>
                                                <Icon type="history" class="is-xs" />
                                                {t('settings.sessions.revoke')}
                                            </SecondaryButton>
                                        )}
                                    </Card>
                                );
                            })}
                        </div>
                    )}

                    {hasMore && <div ref={sentinelRef} class="sentinel" />}
                    {loading && sessions.length > 0 && <LoadingText>{t('settings.sessions.loading')}</LoadingText>}
                </div>
            </SettingsLayout>

            {confirmRevokeId && (
                <ConfirmDialog
                    title={t('settings.sessions.revoke')}
                    message={t('settings.sessions.confirm_revoke')}
                    confirmLabel={t('settings.sessions.revoke')}
                    cancelLabel={t('dialog.cancel')}
                    onConfirm={doRevoke}
                    onClose={() => setConfirmRevokeId(null)}
                />
            )}
        </>
    );
}
