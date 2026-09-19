/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import {
    Badge,
    Button,
    Card,
    CardDescription,
    CardTitle,
    ConfirmDialog,
    DangerButton,
    DeleteOutlineIcon,
    FormMessage,
    HistoryIcon,
    LoadingText,
    Page,
    PageTitle,
    RestoreIcon,
} from 'plaatui';
import { useEffect, useState } from 'preact/hooks';
import { useLocation, useRoute } from 'wouter-preact';
import { type Deployment, type Project } from '../../src-gen/api.ts';
import { AppLayout } from '../components/app-layout.tsx';
import { deleteProject, deployProject, getProject, listDeployments } from '../services/projects.service.ts';

export function ProjectPage() {
    const [, params] = useRoute('/projects/:projectId');
    const [, navigate] = useLocation();
    const id = params?.projectId ?? '';
    const [project, setProject] = useState<Project | null | undefined>(undefined);
    const [deployments, setDeployments] = useState<Deployment[]>([]);
    const [confirmDelete, setConfirmDelete] = useState(false);
    const [error, setError] = useState('');
    const [deploying, setDeploying] = useState(false);
    const [deleting, setDeleting] = useState(false);
    useEffect(() => {
        let active = true;
        let timer: ReturnType<typeof setTimeout>;
        async function refresh() {
            try {
                const nextProject = await getProject(id);
                const nextDeployments = nextProject ? await listDeployments(id) : [];
                if (active) {
                    setProject(nextProject);
                    setDeployments(nextDeployments);
                    setError('');
                }
            } catch {
                if (active) setError('Could not refresh this project.');
            } finally {
                if (active) timer = setTimeout(() => void refresh(), 4000);
            }
        }
        void refresh();
        return () => {
            active = false;
            clearTimeout(timer);
        };
    }, [id]);
    useEffect(() => {
        if (project) document.title = `Plaat Deploy - ${project.name}`;
    }, [project]);
    if (project === undefined)
        return (
            <AppLayout>
                <Page>
                    {error ? <FormMessage type="error" message={error} /> : <LoadingText>Loading project</LoadingText>}
                </Page>
            </AppLayout>
        );
    if (project === null)
        return (
            <AppLayout>
                <Page>
                    <PageTitle>Project not found</PageTitle>
                </Page>
            </AppLayout>
        );
    async function redeploy() {
        setDeploying(true);
        setError('');
        try {
            if (!(await deployProject(id))) setError('Could not queue a deployment.');
        } catch {
            setError('Could not reach the server.');
        } finally {
            setDeploying(false);
        }
    }
    async function removeProject() {
        setDeleting(true);
        setError('');
        try {
            if (await deleteProject(id)) {
                navigate('/');
            } else {
                setError('Could not delete the project.');
            }
        } catch {
            setError('Could not reach the server.');
        } finally {
            setDeleting(false);
            setConfirmDelete(false);
        }
    }
    return (
        <AppLayout>
            <Page size="wide">
                <div class="page-heading">
                    <div>
                        <PageTitle>{project.name}</PageTitle>
                        <p class="subtitle">
                            {project.githubRepo} / {project.githubBranch}
                        </p>
                    </div>
                    <div class="actions">
                        <Button disabled={deploying} onClick={() => void redeploy()}>
                            <RestoreIcon class="is-sm" />
                            {deploying ? 'Queuing deployment' : 'Deploy again'}
                        </Button>
                        <DangerButton disabled={deleting} onClick={() => setConfirmDelete(true)}>
                            <DeleteOutlineIcon class="is-sm" />
                            Delete
                        </DangerButton>
                    </div>
                </div>
                <FormMessage type="error" message={error} />
                <div class="project-overview">
                    <Card>
                        <CardTitle>Status</CardTitle>
                        <p>
                            <Badge accent={project.status === 'running'}>{project.status}</Badge>
                        </p>
                    </Card>
                    <Card>
                        <CardTitle>Type</CardTitle>
                        <p>
                            {project.deploymentMode}
                            {project.containerPort ? ` :${project.containerPort}` : ''}
                        </p>
                        {project.dockerfilePath && <p class="project-detail">{project.dockerfilePath}</p>}
                    </Card>
                    <Card>
                        <CardTitle>Public URL</CardTitle>
                        <a href={project.deploymentUrl} target="_blank" rel="noreferrer">
                            {project.deploymentUrl}
                        </a>
                    </Card>
                </div>
                <div class="section-heading">
                    <HistoryIcon class="is-md" />
                    <h2>Deployments</h2>
                </div>
                <div class="deployment-list">
                    {deployments.map((deployment) => (
                        <Card key={deployment.id}>
                            <div class="card-heading">
                                <div>
                                    <CardTitle tight>
                                        {deployment.commitMessage || deployment.commitSha.slice(0, 8)}
                                    </CardTitle>
                                    <CardDescription>
                                        {deployment.commitSha.slice(0, 12)} ·{' '}
                                        {new Date(deployment.createdAt).toLocaleString()}
                                    </CardDescription>
                                </div>
                                <Badge accent={deployment.status === 'succeeded'}>{deployment.status}</Badge>
                            </div>
                            {deployment.log && (
                                <details>
                                    <summary>Build log</summary>
                                    <pre class="build-log">{deployment.log}</pre>
                                </details>
                            )}
                        </Card>
                    ))}
                </div>
                {confirmDelete && (
                    <ConfirmDialog
                        title="Delete project"
                        message="This removes the container, image, checkout, webhook, and deployment history."
                        confirmLabel="Delete project"
                        cancelLabel="Cancel"
                        confirmText={project.name}
                        onClose={() => setConfirmDelete(false)}
                        onConfirm={() => void removeProject()}
                    />
                )}
            </Page>
        </AppLayout>
    );
}
