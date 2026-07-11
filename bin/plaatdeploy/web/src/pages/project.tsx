/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import { useEffect, useState } from 'preact/hooks';
import { Link, useLocation } from 'wouter-preact';
import { AppLayout } from '../components/app-layout.tsx';
import { ConfirmDialog, Dialog } from '../components/dialog.tsx';
import { RepositoryInput } from '../components/repository-input.tsx';
import { Button, DangerButton, SecondaryButton } from '../components/button.tsx';
import { Card } from '../components/card.tsx';
import { EmptyState } from '../components/empty-state.tsx';
import { FormField } from '../components/form.tsx';
import { FormInput, FormSelect } from '../components/input.tsx';
import {
    ArrowLeftIcon,
    CloudUploadIcon,
    DeleteIcon,
    OpenInNewIcon,
    PencilIcon,
    RocketIcon,
} from '../components/icons.tsx';
import type {
    Deployment,
    DeploymentIndexResponse,
    Project,
    Team,
    TeamIndexResponse,
    TeamGithubStatusResponse,
} from '../src-gen/api.ts';
import { API_URL, authFetch } from '../services/auth.ts';
import { jsonOrThrow } from '../services/api.ts';
import { loadTeamGithubBranches, loadTeamGithubRepositories } from '../services/github.ts';
import { capitalizeLabel, useDocumentTitle } from '../utils.ts';
import { DEPLOY_STATUS_LABELS, STATUS_LABELS } from '../constants.ts';
import { $currentTeamId } from '../services/current-team.ts';

const ACTIVE_DEPLOY_STATUSES = ['pending', 'building'];

interface Props {
    params: { id: string };
}

export function ProjectPage({ params }: Props) {
    const currentTeamId = $currentTeamId.value;
    const [project, setProject] = useState<Project | null>(null);
    const [deployments, setDeployments] = useState<Deployment[]>([]);
    const [loading, setLoading] = useState(true);
    const [loadError, setLoadError] = useState('');
    const [actionError, setActionError] = useState('');
    const [teams, setTeams] = useState<Team[]>([]);
    const [repositories, setRepositories] = useState<string[]>([]);
    const [branches, setBranches] = useState<string[]>([]);
    const [teamGithubStatus, setTeamGithubStatus] = useState<TeamGithubStatusResponse | null>(null);
    const [deploying, setDeploying] = useState(false);
    const [editing, setEditing] = useState(false);
    const [form, setForm] = useState({
        name: '',
        team_id: '',
        github_repo: '',
        github_branch: '',
        base_dir: '',
        container_port: '',
    });
    const [deleteConfirm, setDeleteConfirm] = useState(false);
    const [, navigate] = useLocation();
    useDocumentTitle(project ? project.name : 'Project');

    useEffect(() => {
        let ignore = false;
        setLoading(true);
        setLoadError('');
        setProject(null);
        setDeployments([]);
        (async () => {
            try {
                const [proj, deps, teamData] = await Promise.all([
                    authFetch(`${API_URL}/projects/${params.id}`).then((r) => jsonOrThrow<Project>(r)),
                    authFetch(`${API_URL}/projects/${params.id}/deployments`).then((r) =>
                        jsonOrThrow<DeploymentIndexResponse>(r),
                    ),
                    authFetch(`${API_URL}/teams`).then((r) => jsonOrThrow<TeamIndexResponse>(r)),
                ]);
                if (ignore) return;
                if (currentTeamId && proj.teamId !== currentTeamId) {
                    navigate('/');
                    return;
                }
                setProject(proj);
                setDeployments(deps.data);
                setTeams(teamData.data);
                setForm({
                    name: proj.name,
                    team_id: proj.teamId,
                    github_repo: proj.githubRepo,
                    github_branch: proj.githubBranch,
                    base_dir: proj.baseDir,
                    container_port: proj.containerPort?.toString() ?? '',
                });
            } catch {
                if (!ignore) setLoadError('Failed to load project.');
            } finally {
                if (!ignore) setLoading(false);
            }
        })();
        return () => {
            ignore = true;
        };
    }, [currentTeamId, params.id]);

    useEffect(() => {
        if (!form.team_id) {
            setRepositories(project?.githubRepo ? [project.githubRepo] : []);
            setBranches(project?.githubBranch ? [project.githubBranch] : []);
            setTeamGithubStatus(null);
            return;
        }

        let ignore = false;
        loadTeamGithubRepositories(form.team_id).then(({ repositories, status }) => {
            if (ignore) return;
            const nextRepositories = Array.from(
                new Set([...repositories, ...(project?.githubRepo ? [project.githubRepo] : [])]),
            );
            setTeamGithubStatus(status);
            setRepositories(nextRepositories);
            setForm((prev) => ({
                ...prev,
                github_repo: nextRepositories.includes(prev.github_repo)
                    ? prev.github_repo
                    : (project?.githubRepo ?? ''),
            }));
        });
        return () => {
            ignore = true;
        };
    }, [form.team_id, project?.githubRepo]);

    useEffect(() => {
        if (!form.team_id || !form.github_repo) {
            setBranches(project?.githubBranch ? [project.githubBranch] : []);
            return;
        }

        let ignore = false;
        loadTeamGithubBranches(form.team_id, form.github_repo).then((nextBranches) => {
            if (ignore) return;
            const mergedBranches = Array.from(
                new Set([...nextBranches, ...(project?.githubBranch ? [project.githubBranch] : [])]),
            );
            setBranches(mergedBranches);
            setForm((prev) => ({
                ...prev,
                github_branch: mergedBranches.includes(prev.github_branch)
                    ? prev.github_branch
                    : (project?.githubBranch ?? mergedBranches[0] ?? ''),
            }));
        });
        return () => {
            ignore = true;
        };
    }, [form.github_repo, form.team_id, project?.githubBranch]);

    useEffect(() => {
        const hasActiveDeployment = deployments.some((deployment) =>
            ACTIVE_DEPLOY_STATUSES.includes(deployment.status),
        );
        if (!hasActiveDeployment) return;

        let ignore = false;
        const interval = setInterval(async () => {
            try {
                const deps = await authFetch(`${API_URL}/projects/${params.id}/deployments`).then((r) =>
                    jsonOrThrow<DeploymentIndexResponse>(r),
                );
                const proj = await authFetch(`${API_URL}/projects/${params.id}`).then((r) => jsonOrThrow<Project>(r));
                if (ignore) return;
                setDeployments(deps.data);
                setProject(proj);
            } catch {
                // Ignore polling errors, they will be retried on the next tick.
            }
        }, 3000);
        return () => {
            ignore = true;
            clearInterval(interval);
        };
    }, [deployments, params.id]);

    async function handleDeploy() {
        setActionError('');
        setDeploying(true);
        const res = await authFetch(`${API_URL}/projects/${params.id}/deploy`, { method: 'POST' });
        if (res.ok) {
            const dep: Deployment = await res.json();
            setDeployments((prev) => [dep, ...prev]);
        } else {
            setActionError('Failed to start deployment.');
        }
        setDeploying(false);
    }

    async function handleUpdate(e: Event) {
        e.preventDefault();
        setActionError('');
        const body = new URLSearchParams({
            name: form.name,
            team_id: form.team_id,
            github_repo: form.github_repo,
            github_branch: form.github_branch,
            base_dir: form.base_dir,
        });
        if (form.container_port) body.set('container_port', form.container_port);

        const res = await authFetch(`${API_URL}/projects/${params.id}`, { method: 'PUT', body });
        if (res.ok) {
            const updated: Project = await res.json();
            setProject(updated);
            setEditing(false);
        } else {
            setActionError('Failed to update project.');
        }
    }

    async function handleDelete() {
        setActionError('');
        const res = await authFetch(`${API_URL}/projects/${params.id}`, { method: 'DELETE' });
        if (res.ok) {
            navigate('/');
        } else {
            setActionError('Failed to delete project.');
        }
    }

    if (loading) {
        return (
            <AppLayout>
                <div class="page">
                    <div class="loading">Loading…</div>
                </div>
            </AppLayout>
        );
    }
    if (!project) {
        return (
            <AppLayout>
                <div class="page">
                    {loadError ? (
                        <div class="notification is-danger">{loadError}</div>
                    ) : (
                        <EmptyState icon={<RocketIcon />} message="Project not found." />
                    )}
                </div>
            </AppLayout>
        );
    }

    return (
        <AppLayout>
            <div class="page">
                <div class="page-header">
                    <Link href="/" class="back-link">
                        <ArrowLeftIcon class="is-sm" />
                        Projects
                    </Link>
                    <h1>{project.name}</h1>
                    <span class={`tag is-${project.status}`}>{STATUS_LABELS[project.status]}</span>
                    <span class="spacer" />
                    <div class="buttons">
                        <Button class="is-small" onClick={handleDeploy} disabled={deploying}>
                            <CloudUploadIcon class="is-sm" />
                            {deploying ? 'Deploying…' : 'Deploy'}
                        </Button>
                        <SecondaryButton class="is-small" onClick={() => setEditing(true)}>
                            <PencilIcon class="is-sm" />
                            Edit
                        </SecondaryButton>
                        <DangerButton class="is-small" onClick={() => setDeleteConfirm(true)}>
                            <DeleteIcon class="is-sm" />
                            Delete
                        </DangerButton>
                    </div>
                </div>

                {actionError && <div class="notification is-danger">{actionError}</div>}

                {deleteConfirm && (
                    <ConfirmDialog
                        title="Delete Project"
                        message={`Delete project "${project.name}"?`}
                        confirmLabel="Delete"
                        confirmationText={project.name}
                        onConfirm={handleDelete}
                        onClose={() => setDeleteConfirm(false)}
                    />
                )}

                {editing && (
                    <Dialog title="Edit Project" onClose={() => setEditing(false)}>
                        <form onSubmit={handleUpdate}>
                            <FormField id="edit-project-name" label="Name">
                                <FormInput
                                    id="edit-project-name"
                                    value={form.name}
                                    onInput={(e) => setForm({ ...form, name: (e.target as HTMLInputElement).value })}
                                    required
                                />
                            </FormField>
                            <FormField id="edit-project-repo" label="GitHub Repo">
                                <RepositoryInput
                                    id="edit-project-repo"
                                    repositories={repositories}
                                    value={form.github_repo}
                                    onChange={(github_repo) => setForm({ ...form, github_repo })}
                                    disabled={!teamGithubStatus?.connected}
                                    placeholder="Type to find a repository"
                                />
                            </FormField>
                            <FormField id="edit-project-branch" label="Branch">
                                <FormSelect
                                    id="edit-project-branch"
                                    value={form.github_branch}
                                    onChange={(e) =>
                                        setForm({ ...form, github_branch: (e.target as HTMLSelectElement).value })
                                    }
                                    disabled={!form.github_repo || branches.length === 0}
                                    required
                                >
                                    <option value="" disabled>
                                        {!form.github_repo
                                            ? 'Select a repository first'
                                            : branches.length === 0
                                              ? 'No branches available'
                                              : 'Select a branch'}
                                    </option>
                                    {branches.map((branch) => (
                                        <option key={branch} value={branch}>
                                            {branch}
                                        </option>
                                    ))}
                                </FormSelect>
                            </FormField>
                            <FormField id="edit-project-base-dir" label="Base Dir">
                                <FormInput
                                    id="edit-project-base-dir"
                                    value={form.base_dir}
                                    onInput={(e) =>
                                        setForm({ ...form, base_dir: (e.target as HTMLInputElement).value })
                                    }
                                    placeholder="apps/my-app"
                                />
                            </FormField>
                            <FormField
                                id="edit-project-container-port"
                                label="Container Port (override, auto-detected from EXPOSE)"
                            >
                                <FormInput
                                    id="edit-project-container-port"
                                    type="number"
                                    value={form.container_port}
                                    onInput={(e) =>
                                        setForm({ ...form, container_port: (e.target as HTMLInputElement).value })
                                    }
                                    placeholder="auto"
                                />
                            </FormField>
                            <div class="buttons">
                                <Button type="submit">Save</Button>
                                <SecondaryButton type="button" onClick={() => setEditing(false)}>
                                    Cancel
                                </SecondaryButton>
                            </div>
                        </form>
                    </Dialog>
                )}

                <Card>
                    <div class="card-header">
                        <h2>Details</h2>
                    </div>
                    <div class="table-wrap">
                        <table class="table">
                            <tbody>
                                <tr>
                                    <td>
                                        <strong>Team</strong>
                                    </td>
                                    <td>{teams.find((team) => team.id === project.teamId)?.name ?? 'Unknown team'}</td>
                                </tr>
                                <tr>
                                    <td>
                                        <strong>Repo</strong>
                                    </td>
                                    <td>
                                        <a
                                            class="icon-text"
                                            href={`https://github.com/${project.githubRepo}`}
                                            target="_blank"
                                            rel="noreferrer"
                                        >
                                            {project.githubRepo}
                                            <OpenInNewIcon class="is-sm" />
                                        </a>
                                    </td>
                                </tr>
                                <tr>
                                    <td>
                                        <strong>Branch</strong>
                                    </td>
                                    <td>{project.githubBranch}</td>
                                </tr>
                                {project.baseDir && (
                                    <tr>
                                        <td>
                                            <strong>Base Dir</strong>
                                        </td>
                                        <td>{project.baseDir}</td>
                                    </tr>
                                )}
                                <tr>
                                    <td>
                                        <strong>Build Type</strong>
                                    </td>
                                    <td>{capitalizeLabel(project.buildType)}</td>
                                </tr>
                                {project.containerPort && (
                                    <tr>
                                        <td>
                                            <strong>Container Port</strong>
                                        </td>
                                        <td>{project.containerPort}</td>
                                    </tr>
                                )}
                            </tbody>
                        </table>
                    </div>
                </Card>

                <Card>
                    <div class="card-header">
                        <h2>Deployments</h2>
                    </div>
                    {deployments.length === 0 ? (
                        <EmptyState icon={<CloudUploadIcon />} message="No deployments yet." />
                    ) : (
                        <div class="table-wrap">
                            <table class="table">
                                <thead>
                                    <tr>
                                        <th>Status</th>
                                        <th>Commit</th>
                                        <th>Message</th>
                                        <th>Created</th>
                                        <th></th>
                                    </tr>
                                </thead>
                                <tbody>
                                    {deployments.map((deployment) => (
                                        <tr key={deployment.id}>
                                            <td>
                                                <span class={`tag is-${deployment.status}`}>
                                                    {DEPLOY_STATUS_LABELS[deployment.status]}
                                                </span>
                                            </td>
                                            <td>
                                                <a
                                                    href={`https://github.com/${project.githubRepo}/commit/${deployment.commitSha}`}
                                                    target="_blank"
                                                    rel="noreferrer"
                                                >
                                                    <code>{deployment.commitSha.slice(0, 7)}</code>
                                                </a>
                                            </td>
                                            <td>{deployment.commitMessage}</td>
                                            <td class="has-text-muted">
                                                {new Date(deployment.createdAt).toLocaleString()}
                                            </td>
                                            <td>
                                                <Link href={`/deployments/${deployment.id}`}>View</Link>
                                            </td>
                                        </tr>
                                    ))}
                                </tbody>
                            </table>
                        </div>
                    )}
                </Card>
            </div>
        </AppLayout>
    );
}
