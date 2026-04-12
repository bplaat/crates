/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import { useEffect, useState } from 'preact/hooks';
import { Link } from 'wouter-preact';
import { AppLayout } from '../components/app-layout.tsx';
import type { Deployment } from '../src-gen/api.ts';
import { API_URL, authFetch } from '../services/auth.ts';

const STATUS_LABELS: Record<string, string> = {
    pending: 'Pending',
    building: 'Building',
    succeeded: 'Succeeded',
    failed: 'Failed',
};

interface Props {
    params: { id: string };
}

export function DeploymentPage({ params }: Props) {
    const [deployment, setDeployment] = useState<Deployment | null>(null);
    const [loading, setLoading] = useState(true);

    useEffect(() => {
        authFetch(`${API_URL}/deployments/${params.id}`)
            .then((r) => r.json())
            .then((d: Deployment) => {
                setDeployment(d);
                setLoading(false);
            });
    }, [params.id]);

    if (loading) {
        return (
            <AppLayout>
                <div class="page">
                    <div class="loading">Loading...</div>
                </div>
            </AppLayout>
        );
    }
    if (!deployment) {
        return (
            <AppLayout>
                <div class="page">
                    <div class="empty">Deployment not found.</div>
                </div>
            </AppLayout>
        );
    }

    return (
        <AppLayout>
            <div class="page">
                <div class="page-header">
                    <Link href={`/projects/${deployment.projectId}`}>&#8592; Project</Link>
                    <h1>Deployment</h1>
                    <span class={`badge badge-${deployment.status}`}>{STATUS_LABELS[deployment.status]}</span>
                </div>
                <div class="card">
                    <table>
                        <tbody>
                            <tr>
                                <td>
                                    <strong>Commit</strong>
                                </td>
                                <td>
                                    <code>{deployment.commitSha.slice(0, 7)}</code> {deployment.commitMessage}
                                </td>
                            </tr>
                            <tr>
                                <td>
                                    <strong>Status</strong>
                                </td>
                                <td>{STATUS_LABELS[deployment.status]}</td>
                            </tr>
                            <tr>
                                <td>
                                    <strong>Date</strong>
                                </td>
                                <td>{new Date(deployment.createdAt).toLocaleString()}</td>
                            </tr>
                        </tbody>
                    </table>
                </div>
                <div class="page-header" style="margin-top: 16px;">
                    <h1 style="font-size: 16px;">Build Log</h1>
                </div>
                <div class="log-view">{deployment.log ?? '(no log yet)'}</div>
            </div>
        </AppLayout>
    );
}
