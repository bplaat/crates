/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import { useEffect, useState } from 'preact/hooks';
import { Link, useLocation } from 'wouter-preact';
import { AppLayout } from '../components/app-layout.tsx';
import { ConfirmDialog, Dialog } from '../components/dialog.tsx';
import type {
    Deployment,
    DeploymentIndexResponse,
    Project,
    Team,
    TeamIndexResponse,
    TeamGithubStatusResponse,
} from '../src-gen/api.ts';
import { API_URL, authFetch } from '../services/auth.ts';
import { loadTeamGithubBranches, loadTeamGithubRepositories } from '../services/github.ts';
import { capitalizeLabel } from '../utils.ts';

const STATUS_LABELS: Record<string, string> = {
    idle: 'Idle',
    building: 'Building',
    running: 'Running',
    failed: 'Failed',
};

const DEPLOY_STATUS_LABELS: Record<string, string> = {
    pending: 'Pending',
    building: 'Building',
    succeeded: 'Succeeded',
    failed: 'Failed',
};

interface Props {
    params: { id: string };
}

export function ProjectPage({ params }: Props) {
    const [project, setProject] = useState<Project | null>(null);
    const [deployments, setDeployments] = useState<Deployment[]>([]);
    const [loading, setLoading] = useState(true);
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

    useEffect(() => {
        Promise.all([
            authFetch(`${API_URL}/projects/${params.id}`).then((r) => r.json()),
            authFetch(`${API_URL}/projects/${params.id}/deployments`).then((r) => r.json()),
            authFetch(`${API_URL}/teams`).then((r) => r.json()),
        ]).then(([proj, deps, teamData]: [Project, DeploymentIndexResponse, TeamIndexResponse]) => {
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
            setLoading(false);
        });
    }, [params.id]);

    useEffect(() => {
        if (!form.team_id) {
            setRepositories(project?.githubRepo ? [project.githubRepo] : []);
            setBranches(project?.githubBranch ? [project.githubBranch] : []);
            setTeamGithubStatus(null);
            return;
        }

        loadTeamGithubRepositories(form.team_id).then(({ repositories, status }) => {
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
    }, [form.team_id, project?.githubRepo]);

    useEffect(() => {
        if (!form.team_id || !form.github_repo) {
            setBranches(project?.githubBranch ? [project.githubBranch] : []);
            return;
        }

        loadTeamGithubBranches(form.team_id, form.github_repo).then((nextBranches) => {
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
    }, [form.github_repo, form.team_id, project?.githubBranch]);

    async function handleDeploy() {
        setDeploying(true);
        const res = await authFetch(`${API_URL}/projects/${params.id}/deploy`, { method: 'POST' });
        if (res.ok) {
            const dep: Deployment = await res.json();
            setDeployments((prev) => [dep, ...prev]);
        }
        setDeploying(false);
    }

    async function handleUpdate(e: Event) {
        e.preventDefault();
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
        }
    }

    async function handleDelete() {
        const res = await authFetch(`${API_URL}/projects/${params.id}`, { method: 'DELETE' });
        if (res.ok) navigate('/');
    }

    if (loading) {
        return (
            <AppLayout>
                <div class="page">
                    <div class="loading">Loading...</div>
                </div>
            </AppLayout>
        );
    }
    if (!project) {
        return (
            <AppLayout>
                <div class="page">
                    <div class="empty">Project not found.</div>
                </div>
            </AppLayout>
        );
    }

    return (
        <AppLayout>
            <div class="page">
                <div class="page-header">
                    <Link href="/">&#8592; Home</Link>
                    <h1>{project.name}</h1>
                    <span class={`badge badge-${project.status}`}>{STATUS_LABELS[project.status]}</span>
                    <span class="spacer" />
                    <div class="section-actions">
                        <button class="btn btn-primary btn-sm" onClick={handleDeploy} disabled={deploying}>
                            {deploying ? 'Deploying...' : 'Deploy'}
                        </button>
                        <button class="btn btn-secondary btn-sm" onClick={() => setEditing(true)}>
                            Edit
                        </button>
                        <button class="btn btn-danger btn-sm" onClick={() => setDeleteConfirm(true)}>
                            Delete
                        </button>
                    </div>
                </div>

                {deleteConfirm && (
                    <ConfirmDialog
                        title="Delete Project"
                        message={`Delete project "${project.name}"?`}
                        confirmLabel="Delete"
                        onConfirm={handleDelete}
                        onClose={() => setDeleteConfirm(false)}
                    />
                )}

                {editing && (
                    <Dialog title="Edit Project" onClose={() => setEditing(false)}>
                        <form onSubmit={handleUpdate}>
                            <div class="form-group">
                                <label>Team</label>
                                <select
                                    value={form.team_id}
                                    onChange={(e) =>
                                        setForm({ ...form, team_id: (e.target as HTMLSelectElement).value })
                                    }
                                    required
                                >
                                    {teams.map((team) => (
                                        <option key={team.id} value={team.id}>
                                            {team.name}
                                        </option>
                                    ))}
                                </select>
                            </div>
                            <div class="form-group">
                                <label>Name</label>
                                <input
                                    value={form.name}
                                    onInput={(e) => setForm({ ...form, name: (e.target as HTMLInputElement).value })}
                                    required
                                />
                            </div>
                            <div class="form-group">
                                <label>GitHub Repo</label>
                                <select
                                    value={form.github_repo}
                                    onChange={(e) =>
                                        setForm({ ...form, github_repo: (e.target as HTMLSelectElement).value })
                                    }
                                    disabled={!teamGithubStatus?.connected && form.team_id !== project.teamId}
                                    required
                                >
                                    <option value="" disabled>
                                        {!form.team_id
                                            ? 'Select a team first'
                                            : !teamGithubStatus?.appConfigured
                                              ? 'GitHub App is not configured yet'
                                              : !teamGithubStatus.connected && form.team_id !== project.teamId
                                                ? 'Connect GitHub for this team on the Teams page'
                                                : 'Select a repository'}
                                    </option>
                                    {repositories.map((repository) => (
                                        <option key={repository} value={repository}>
                                            {repository}
                                        </option>
                                    ))}
                                </select>
                                {!teamGithubStatus?.connected && form.team_id !== project.teamId && (
                                    <small class="muted-text">
                                        Connect GitHub for the selected team on the Teams page before moving this
                                        project.
                                    </small>
                                )}
                            </div>
                            <div class="form-group">
                                <label>Branch</label>
                                <select
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
                                </select>
                            </div>
                            <div class="form-group">
                                <label>Base Dir</label>
                                <input
                                    value={form.base_dir}
                                    onInput={(e) =>
                                        setForm({ ...form, base_dir: (e.target as HTMLInputElement).value })
                                    }
                                    placeholder="apps/my-app"
                                />
                            </div>
                            <div class="form-group">
                                <label>Container Port (override, auto-detected from EXPOSE)</label>
                                <input
                                    type="number"
                                    value={form.container_port}
                                    onInput={(e) =>
                                        setForm({ ...form, container_port: (e.target as HTMLInputElement).value })
                                    }
                                    placeholder="auto"
                                />
                            </div>
                            <div class="dialog-actions">
                                <button class="btn btn-primary" type="submit">
                                    Save
                                </button>
                                <button class="btn btn-secondary" type="button" onClick={() => setEditing(false)}>
                                    Cancel
                                </button>
                            </div>
                        </form>
                    </Dialog>
                )}

                <div class="card">
                    <h2>Details</h2>
                    <table style="margin-top: 8px;">
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
                                <td>{project.githubRepo}</td>
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

                <div class="card">
                    <h2>Deployments</h2>
                    {deployments.length === 0 ? (
                        <div class="empty">No deployments yet.</div>
                    ) : (
                        <table style="margin-top: 8px;">
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
                                            <span class={`badge badge-${deployment.status}`}>
                                                {DEPLOY_STATUS_LABELS[deployment.status]}
                                            </span>
                                        </td>
                                        <td>{deployment.commitSha}</td>
                                        <td>{deployment.commitMessage}</td>
                                        <td>{new Date(deployment.createdAt).toLocaleString()}</td>
                                        <td>
                                            <Link href={`/deployments/${deployment.id}`}>View</Link>
                                        </td>
                                    </tr>
                                ))}
                            </tbody>
                        </table>
                    )}
                </div>
            </div>
        </AppLayout>
    );
}
