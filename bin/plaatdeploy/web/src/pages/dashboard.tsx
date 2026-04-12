/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import { useEffect, useMemo, useState } from 'preact/hooks';
import { useLocation } from 'wouter-preact';
import { AppLayout } from '../components/app-layout.tsx';
import { Dialog } from '../components/dialog.tsx';
import type {
    Project,
    ProjectIndexResponse,
    Team,
    TeamIndexResponse,
    TeamGithubStatusResponse,
} from '../src-gen/api.ts';
import { $authUser, API_URL, authFetch } from '../services/auth.ts';
import { loadTeamGithubBranches, loadTeamGithubRepositories } from '../services/github.ts';

const STATUS_LABELS: Record<string, string> = {
    idle: 'Idle',
    building: 'Building',
    running: 'Running',
    failed: 'Failed',
};

export function DashboardPage() {
    const [projects, setProjects] = useState<Project[]>([]);
    const [teams, setTeams] = useState<Team[]>([]);
    const [repositories, setRepositories] = useState<string[]>([]);
    const [branches, setBranches] = useState<string[]>([]);
    const [loading, setLoading] = useState(true);
    const [, navigate] = useLocation();
    const authUser = $authUser.value;
    const [teamGithubStatus, setTeamGithubStatus] = useState<TeamGithubStatusResponse | null>(null);

    const [creating, setCreating] = useState(false);
    const [form, setForm] = useState({ name: '', github_repo: '', github_branch: '', base_dir: '', team_id: '' });
    const [createError, setCreateError] = useState('');

    useEffect(() => {
        Promise.all([
            authFetch(`${API_URL}/projects`).then((r) => r.json()),
            authFetch(`${API_URL}/teams`).then((r) => r.json()),
        ]).then(([projectData, teamData]: [ProjectIndexResponse, TeamIndexResponse]) => {
            setProjects(projectData.data);
            setTeams(teamData.data);
            setForm((prev) => ({ ...prev, team_id: teamData.data[0]?.id ?? '' }));
            setLoading(false);
        });
    }, []);

    useEffect(() => {
        if (!form.team_id) {
            setRepositories([]);
            setBranches([]);
            setTeamGithubStatus(null);
            setForm((prev) => ({ ...prev, github_repo: '', github_branch: '' }));
            return;
        }

        loadTeamGithubRepositories(form.team_id).then(({ repositories, status }) => {
            setTeamGithubStatus(status);
            setRepositories(repositories);
            setForm((prev) => ({
                ...prev,
                github_repo: repositories.includes(prev.github_repo) ? prev.github_repo : '',
                github_branch: repositories.includes(prev.github_repo) ? prev.github_branch : '',
            }));
        });
    }, [form.team_id]);

    useEffect(() => {
        if (!form.team_id || !form.github_repo) {
            setBranches([]);
            setForm((prev) => ({ ...prev, github_branch: prev.github_repo ? prev.github_branch : '' }));
            return;
        }

        loadTeamGithubBranches(form.team_id, form.github_repo).then((nextBranches) => {
            setBranches(nextBranches);
            setForm((prev) => ({
                ...prev,
                github_branch: nextBranches.includes(prev.github_branch) ? prev.github_branch : (nextBranches[0] ?? ''),
            }));
        });
    }, [form.github_repo, form.team_id]);

    async function handleCreate(e: Event) {
        e.preventDefault();
        setCreateError('');
        const body = new URLSearchParams({
            name: form.name,
            github_repo: form.github_repo,
            github_branch: form.github_branch,
            base_dir: form.base_dir,
            team_id: form.team_id,
        });

        const res = await authFetch(`${API_URL}/projects`, { method: 'POST', body });
        if (!res.ok) {
            setCreateError('Failed to create project.');
            return;
        }
        const project: Project = await res.json();
        setProjects((prev) => [project, ...prev]);
        setCreating(false);
        setForm({ name: '', github_repo: '', github_branch: '', base_dir: '', team_id: form.team_id });
        setBranches([]);
    }

    const projectsByTeam = useMemo(() => {
        return new Map(teams.map((team) => [team.id, projects.filter((project) => project.teamId === team.id)]));
    }, [projects, teams]);

    return (
        <AppLayout>
            <div class="page">
                <div class="page-header">
                    <div>
                        <h1>Home</h1>
                        <p class="page-subtitle">
                            {authUser?.role === 'admin'
                                ? 'All teams and their projects.'
                                : 'Your teams and their projects.'}
                        </p>
                    </div>
                    <span class="spacer" />
                    <button class="btn btn-primary btn-sm" onClick={() => setCreating(true)}>
                        + New Project
                    </button>
                </div>

                {creating && (
                    <Dialog title="New Project" onClose={() => setCreating(false)}>
                        {createError && <div class="alert alert-error">{createError}</div>}
                        <form onSubmit={handleCreate}>
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
                                <label>Name (subdomain slug)</label>
                                <input
                                    value={form.name}
                                    onInput={(e) => setForm({ ...form, name: (e.target as HTMLInputElement).value })}
                                    placeholder="my-app"
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
                                    disabled={!teamGithubStatus?.connected}
                                    required
                                >
                                    <option value="" disabled>
                                        {!form.team_id
                                            ? 'Select a team first'
                                            : !teamGithubStatus?.appConfigured
                                              ? 'GitHub App is not configured yet'
                                              : !teamGithubStatus.connected
                                                ? 'Connect GitHub for this team on the Teams page'
                                                : repositories.length === 0
                                                  ? 'No repositories available'
                                                  : 'Select a repository'}
                                    </option>
                                    {repositories.map((repository) => (
                                        <option key={repository} value={repository}>
                                            {repository}
                                        </option>
                                    ))}
                                </select>
                                {form.team_id && teamGithubStatus?.connected === false && (
                                    <small class="muted-text">
                                        Connect GitHub for the selected team on the Teams page before creating a
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
                                <label>Base Dir (optional, path within repo)</label>
                                <input
                                    value={form.base_dir}
                                    onInput={(e) =>
                                        setForm({ ...form, base_dir: (e.target as HTMLInputElement).value })
                                    }
                                    placeholder="apps/my-app"
                                />
                            </div>
                            <div class="dialog-actions">
                                <button class="btn btn-primary" type="submit">
                                    Create
                                </button>
                                <button class="btn btn-secondary" type="button" onClick={() => setCreating(false)}>
                                    Cancel
                                </button>
                            </div>
                        </form>
                    </Dialog>
                )}

                {loading && <div class="loading">Loading projects...</div>}
                {!loading && teams.length === 0 && <div class="empty">No teams available yet.</div>}
                {!loading &&
                    teams.map((team) => {
                        const teamProjects = projectsByTeam.get(team.id) ?? [];

                        return (
                            <div key={team.id} class="card">
                                <div class="page-header" style="margin-bottom: 12px;">
                                    <div>
                                        <h2>{team.name}</h2>
                                        <p class="card-meta">
                                            {teamProjects.length} project{teamProjects.length === 1 ? '' : 's'}
                                        </p>
                                    </div>
                                </div>

                                {teamProjects.length === 0 ? (
                                    <div class="empty" style="margin: 0;">
                                        No projects yet.
                                    </div>
                                ) : (
                                    <div class="project-list">
                                        {teamProjects.map((project) => (
                                            <button
                                                key={project.id}
                                                class="project-list-item"
                                                onClick={() => navigate(`/projects/${project.id}`)}
                                            >
                                                <div>
                                                    <strong>{project.name}</strong>
                                                    <div class="card-meta">
                                                        {project.githubRepo} ({project.githubBranch})
                                                        {project.baseDir ? ` / ${project.baseDir}` : ''}
                                                    </div>
                                                </div>
                                                <span class={`badge badge-${project.status}`}>
                                                    {STATUS_LABELS[project.status]}
                                                </span>
                                            </button>
                                        ))}
                                    </div>
                                )}
                            </div>
                        );
                    })}
            </div>
        </AppLayout>
    );
}
