/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import { useEffect, useState } from 'preact/hooks';
import { Link, useLocation } from 'wouter-preact';
import { AppLayout } from '../components/app-layout.tsx';
import { Card } from '../components/card.tsx';
import { EmptyState } from '../components/empty-state.tsx';
import { ArrowLeftIcon, CloudUploadIcon } from '../components/icons.tsx';
import type { Deployment, Project } from '../src-gen/api.ts';
import { API_URL, authFetch } from '../services/auth.ts';
import { jsonOrThrow } from '../services/api.ts';
import { DEPLOY_STATUS_LABELS } from '../constants.ts';
import { $currentTeamId } from '../services/current-team.ts';
import { useDocumentTitle } from '../utils.ts';

const ACTIVE_DEPLOY_STATUSES = ['pending', 'building'];

interface Props {
    params: { id: string };
}

export function DeploymentPage({ params }: Props) {
    const currentTeamId = $currentTeamId.value;
    const [, navigate] = useLocation();
    const [deployment, setDeployment] = useState<Deployment | null>(null);
    const [project, setProject] = useState<Project | null>(null);
    const [loading, setLoading] = useState(true);
    const [loadError, setLoadError] = useState('');
    useDocumentTitle('Deployment');

    useEffect(() => {
        let ignore = false;
        setLoading(true);
        setLoadError('');
        setDeployment(null);
        setProject(null);
        (async () => {
            try {
                const d = await authFetch(`${API_URL}/deployments/${params.id}`).then((r) =>
                    jsonOrThrow<Deployment>(r),
                );
                if (ignore) return;
                setDeployment(d);
                const p = await authFetch(`${API_URL}/projects/${d.projectId}`).then((r) => jsonOrThrow<Project>(r));
                if (ignore) return;
                if (currentTeamId && p.teamId !== currentTeamId) {
                    navigate('/');
                    return;
                }
                setProject(p);
            } catch {
                if (!ignore) setLoadError('Failed to load deployment.');
            } finally {
                if (!ignore) setLoading(false);
            }
        })();
        return () => {
            ignore = true;
        };
    }, [currentTeamId, params.id]);

    useEffect(() => {
        if (!deployment || !ACTIVE_DEPLOY_STATUSES.includes(deployment.status)) return;

        let ignore = false;
        const interval = setInterval(async () => {
            try {
                const d = await authFetch(`${API_URL}/deployments/${params.id}`).then((r) =>
                    jsonOrThrow<Deployment>(r),
                );
                if (ignore) return;
                setDeployment(d);
            } catch {
                // Ignore polling errors, they will be retried on the next tick.
            }
        }, 3000);
        return () => {
            ignore = true;
            clearInterval(interval);
        };
    }, [deployment, params.id]);

    if (loading) {
        return (
            <AppLayout>
                <div class="page">
                    <div class="loading">Loading…</div>
                </div>
            </AppLayout>
        );
    }
    if (!deployment) {
        return (
            <AppLayout>
                <div class="page">
                    {loadError ? (
                        <div class="notification is-danger">{loadError}</div>
                    ) : (
                        <EmptyState icon={<CloudUploadIcon />} message="Deployment not found." />
                    )}
                </div>
            </AppLayout>
        );
    }

    return (
        <AppLayout>
            <div class="page">
                <div class="page-header">
                    <Link href={`/projects/${deployment.projectId}`} class="back-link">
                        <ArrowLeftIcon class="is-sm" />
                        Project
                    </Link>
                    <h1>Deployment</h1>
                    <span class={`tag is-${deployment.status}`}>{DEPLOY_STATUS_LABELS[deployment.status]}</span>
                </div>

                <Card>
                    <div class="table-wrap">
                        <table class="table">
                            <tbody>
                                <tr>
                                    <td>
                                        <strong>Commit</strong>
                                    </td>
                                    <td>
                                        {project ? (
                                            <a
                                                href={`https://github.com/${project.githubRepo}/commit/${deployment.commitSha}`}
                                                target="_blank"
                                                rel="noreferrer"
                                            >
                                                <code>{deployment.commitSha.slice(0, 7)}</code>
                                            </a>
                                        ) : (
                                            <code>{deployment.commitSha.slice(0, 7)}</code>
                                        )}{' '}
                                        {deployment.commitMessage}
                                    </td>
                                </tr>
                                <tr>
                                    <td>
                                        <strong>Status</strong>
                                    </td>
                                    <td>{DEPLOY_STATUS_LABELS[deployment.status]}</td>
                                </tr>
                                <tr>
                                    <td>
                                        <strong>Date</strong>
                                    </td>
                                    <td class="has-text-muted">{new Date(deployment.createdAt).toLocaleString()}</td>
                                </tr>
                            </tbody>
                        </table>
                    </div>
                </Card>

                <div class="card-header">
                    <h2>Build Log</h2>
                </div>
                <div class="log-view">{deployment.log ?? '(no log yet)'}</div>
            </div>
        </AppLayout>
    );
}
