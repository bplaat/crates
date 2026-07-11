/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import { useEffect, useMemo, useState } from 'preact/hooks';
import type { Session } from '../src-gen/api.ts';
import { ConfirmDialog } from '../components/dialog.tsx';
import { SettingsLayout } from '../components/settings-layout.tsx';
import { Card } from '../components/card.tsx';
import { EmptyState } from '../components/empty-state.tsx';
import { SecondaryButton } from '../components/button.tsx';
import { LaptopIcon } from '../components/icons.tsx';
import { $authUser, $currentSessionId } from '../services/auth.ts';
import { listSessions, revokeSession } from '../services/sessions.ts';
import { useDocumentTitle } from '../utils.ts';

function clientLabel(session: Session): string {
    const { name, version, os } = session.client;
    if (name && version && os) return `${name} ${version} on ${os}`;
    if (name && version) return `${name} ${version}`;
    if (name && os) return `${name} on ${os}`;
    if (name) return name;
    return 'Unknown client';
}

function locationLabel(session: Session): string {
    const { address, city, country } = session.ip;
    const place = [city, country].filter(Boolean).join(', ');
    if (address && place) return `${address} (${place})`;
    if (address) return address;
    if (place) return place;
    return 'Unknown location';
}

export function SettingsSessionsPage() {
    const authUser = $authUser.value!;
    const currentSessionId = $currentSessionId.value;
    const [sessions, setSessions] = useState<Session[]>([]);
    const [loading, setLoading] = useState(true);
    const [loadError, setLoadError] = useState('');
    const [confirmRevokeId, setConfirmRevokeId] = useState<string | null>(null);
    useDocumentTitle('Sessions');

    useEffect(() => {
        listSessions()
            .then((response) => {
                setSessions(response.data);
            })
            .catch(() => {
                setLoadError('Failed to load sessions.');
            })
            .finally(() => {
                setLoading(false);
            });
    }, []);

    const ownSessions = useMemo(() => {
        return sessions.filter((session) => session.userId === authUser.id);
    }, [authUser.id, sessions]);

    async function handleRevoke() {
        if (!confirmRevokeId) return;
        const ok = await revokeSession(confirmRevokeId);
        if (ok) {
            setSessions((current) => current.filter((session) => session.id !== confirmRevokeId));
        }
        setConfirmRevokeId(null);
    }

    return (
        <>
            <SettingsLayout>
                <div class="page">
                    <div class="page-header">
                        <div>
                            <h1>Sessions</h1>
                            <p class="page-subtitle">Review active sessions and revoke old devices.</p>
                        </div>
                    </div>

                    {loadError && <div class="notification is-danger">{loadError}</div>}
                    {loading && <div class="loading">Loading sessions…</div>}
                    {!loading && !loadError && ownSessions.length === 0 && (
                        <EmptyState icon={<LaptopIcon />} message="No active sessions found." />
                    )}
                    {!loading && ownSessions.length > 0 && (
                        <div class="stack is-narrow">
                            {ownSessions.map((session) => {
                                const isCurrent = session.id === currentSessionId;

                                return (
                                    <Card key={session.id}>
                                        <div class="split">
                                            <div>
                                                <div class="inline-row">
                                                    <strong>{clientLabel(session)}</strong>
                                                    {isCurrent && <span class="tag is-running">Current</span>}
                                                </div>
                                                <p class="card-meta" style="margin-top: 6px;">
                                                    {locationLabel(session)}
                                                </p>
                                                <p class="card-meta" style="margin-top: 6px;">
                                                    Created {new Date(session.createdAt).toLocaleString()} - Expires{' '}
                                                    {new Date(session.expiresAt).toLocaleString()}
                                                </p>
                                            </div>
                                            {!isCurrent && (
                                                <SecondaryButton
                                                    class="is-small"
                                                    type="button"
                                                    onClick={() => setConfirmRevokeId(session.id)}
                                                >
                                                    Revoke
                                                </SecondaryButton>
                                            )}
                                        </div>
                                    </Card>
                                );
                            })}
                        </div>
                    )}
                </div>
            </SettingsLayout>

            {confirmRevokeId && (
                <ConfirmDialog
                    title="Revoke Session"
                    message="Revoke this session? The device will be signed out."
                    confirmLabel="Revoke"
                    onConfirm={handleRevoke}
                    onClose={() => setConfirmRevokeId(null)}
                />
            )}
        </>
    );
}
