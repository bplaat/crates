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
    CloudUploadIcon,
    Dialog,
    EmptyState,
    Form,
    FormActions,
    FormField,
    FormInput,
    FormMessage,
    FormSelect,
    LoadingText,
    LockIcon,
    Page,
    PageTitle,
    PlusIcon,
    SearchInput,
    SecondaryButton,
} from 'plaatui';
import { useEffect, useMemo, useState } from 'preact/hooks';
import { useLocation } from 'wouter-preact';
import { type DeploymentMode, type GitHubRepository, type Project } from '../../src-gen/api.ts';
import { AppLayout } from '../components/app-layout.tsx';
import { $authUser } from '../services/auth.service.ts';
import { createProject, listProjects, listRepositories, scanRepository } from '../services/projects.service.ts';

export function Dashboard() {
    const [, navigate] = useLocation();
    const [projects, setProjects] = useState<Project[] | null>(null);
    const [projectsError, setProjectsError] = useState('');
    const [repositories, setRepositories] = useState<GitHubRepository[] | null>(null);
    const [dialog, setDialog] = useState(false);
    const [query, setQuery] = useState('');
    const [error, setError] = useState('');
    const [selectedRepository, setSelectedRepository] = useState<GitHubRepository | null>(null);
    const [deploymentMode, setDeploymentMode] = useState<DeploymentMode>('static');
    const [dockerfilePath, setDockerfilePath] = useState('Dockerfile');
    const [scanning, setScanning] = useState(false);
    const [creating, setCreating] = useState(false);
    const connected = Boolean($authUser.value?.githubLogin);
    useEffect(() => {
        document.title = 'Plaat Deploy';
        void listProjects()
            .then(setProjects)
            .catch(() => {
                setProjects([]);
                setProjectsError('Could not load projects.');
            });
    }, []);
    const filtered = useMemo(
        () => repositories?.filter((repo) => repo.fullName.toLowerCase().includes(query.toLowerCase())),
        [repositories, query],
    );
    async function openCreate() {
        if (!connected) {
            navigate('/settings');
            return;
        }
        setDialog(true);
        setError('');
        setQuery('');
        setSelectedRepository(null);
        setRepositories(null);
        try {
            setRepositories(await listRepositories());
        } catch {
            setError('Could not load GitHub repositories. Check your token permissions.');
        }
    }
    async function choose(repo: GitHubRepository) {
        setSelectedRepository(repo);
        setScanning(true);
        setError('');
        try {
            const scan = await scanRepository(repo.fullName);
            setDeploymentMode(scan.deploymentMode);
            setDockerfilePath(scan.dockerfilePath ?? 'Dockerfile');
        } catch {
            setSelectedRepository(null);
            setError('Could not scan the repository. Try again.');
        } finally {
            setScanning(false);
        }
    }
    async function submit(event: SubmitEvent) {
        event.preventDefault();
        if (!selectedRepository) return;
        setCreating(true);
        setError('');
        try {
            const project = await createProject(
                selectedRepository.fullName,
                deploymentMode,
                deploymentMode === 'dockerfile' ? dockerfilePath : undefined,
            );
            navigate(`/projects/${project.id}`);
        } catch {
            setError('Could not create the project or install its webhook.');
        } finally {
            setCreating(false);
        }
    }
    return (
        <AppLayout>
            <Page size="wide">
                <div class="page-heading">
                    <div>
                        <PageTitle>Projects</PageTitle>
                        <p class="subtitle">Your GitHub repositories, deployed on this server.</p>
                    </div>
                    <Button onClick={() => void openCreate()}>
                        <PlusIcon class="is-sm" />
                        New project
                    </Button>
                </div>
                <FormMessage type="error" message={projectsError} />
                {projects === null ? (
                    <LoadingText>Loading projects</LoadingText>
                ) : projects.length === 0 ? (
                    <EmptyState
                        icon={<CloudUploadIcon class="is-xl" />}
                        message={
                            connected
                                ? 'No projects yet. Deploy your first repository.'
                                : 'Connect GitHub in Settings to create a project.'
                        }
                    />
                ) : (
                    <div class="project-grid">
                        {projects.map((project) => (
                            <a
                                key={project.id}
                                class="project-link"
                                href={`/projects/${project.id}`}
                                onClick={(event) => {
                                    event.preventDefault();
                                    navigate(`/projects/${project.id}`);
                                }}
                            >
                                <Card>
                                    <div class="card-heading">
                                        <CardTitle tight>{project.name}</CardTitle>
                                        <Badge accent={project.status === 'running'}>{project.status}</Badge>
                                    </div>
                                    <CardDescription>{project.githubRepo}</CardDescription>
                                    <div class="project-meta">
                                        <span>{project.deploymentMode}</span>
                                        <span>{project.deploymentUrl}</span>
                                    </div>
                                </Card>
                            </a>
                        ))}
                    </div>
                )}
                {dialog && (
                    <Dialog title="Deploy a GitHub repository" onClose={() => setDialog(false)}>
                        {selectedRepository ? (
                            scanning ? (
                                <LoadingText>Scanning repository</LoadingText>
                            ) : (
                                <Form onSubmit={submit}>
                                    <p class="selected-repository">{selectedRepository.fullName}</p>
                                    <FormField id="deploymentMode" label="Project type">
                                        <FormSelect
                                            id="deploymentMode"
                                            value={deploymentMode}
                                            onChange={(event) =>
                                                setDeploymentMode(
                                                    (event.target as HTMLSelectElement).value as DeploymentMode,
                                                )
                                            }
                                        >
                                            <option value="static">Static files</option>
                                            <option value="dockerfile">Dockerfile</option>
                                        </FormSelect>
                                    </FormField>
                                    {deploymentMode === 'dockerfile' && (
                                        <FormField id="dockerfilePath" label="Dockerfile path">
                                            <FormInput
                                                id="dockerfilePath"
                                                type="text"
                                                required
                                                value={dockerfilePath}
                                                placeholder="Dockerfile"
                                                onInput={(event) =>
                                                    setDockerfilePath((event.target as HTMLInputElement).value)
                                                }
                                            />
                                        </FormField>
                                    )}
                                    <FormMessage type="error" message={error} />
                                    <FormActions flush>
                                        <SecondaryButton
                                            type="button"
                                            disabled={creating}
                                            onClick={() => setSelectedRepository(null)}
                                        >
                                            Back
                                        </SecondaryButton>
                                        <Button type="submit" disabled={creating}>
                                            {creating ? 'Creating project' : 'Create project'}
                                        </Button>
                                    </FormActions>
                                </Form>
                            )
                        ) : (
                            <>
                                <SearchInput
                                    value={query}
                                    onInput={setQuery}
                                    onClear={() => setQuery('')}
                                    placeholder="Search repositories"
                                />
                                <FormMessage type="error" message={error} />
                                {repositories === null && !error ? (
                                    <LoadingText>Loading repositories</LoadingText>
                                ) : (
                                    <div class="repo-list">
                                        {filtered?.map((repo) => (
                                            <button
                                                key={repo.fullName}
                                                type="button"
                                                class="repo-row"
                                                disabled={scanning}
                                                onClick={() => void choose(repo)}
                                            >
                                                <div>
                                                    <strong>{repo.fullName}</strong>
                                                    <span>{repo.defaultBranch}</span>
                                                </div>
                                                {repo.private && <LockIcon class="is-sm" />}
                                            </button>
                                        ))}
                                    </div>
                                )}
                            </>
                        )}
                    </Dialog>
                )}
            </Page>
        </AppLayout>
    );
}
