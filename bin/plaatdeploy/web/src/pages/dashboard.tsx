/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import { useEffect, useMemo, useState } from 'preact/hooks';
import { useLocation } from 'wouter-preact';
import { AppLayout } from '../components/app-layout.tsx';
import { Dialog } from '../components/dialog.tsx';
import { RepositoryInput } from '../components/repository-input.tsx';
import { Button, SecondaryButton } from '../components/button.tsx';
import { Card } from '../components/card.tsx';
import { EmptyState } from '../components/empty-state.tsx';
import { FormField } from '../components/form.tsx';
import { FormInput, FormSelect } from '../components/input.tsx';
import { PlusIcon, RocketIcon } from '../components/icons.tsx';
import type {
    Project,
    ProjectIndexResponse,
    Team,
    TeamIndexResponse,
    TeamGithubStatusResponse,
} from '../src-gen/api.ts';
import { API_URL, authFetch } from '../services/auth.ts';
import { jsonOrThrow } from '../services/api.ts';
import { loadTeamGithubBranches, loadTeamGithubRepositories } from '../services/github.ts';
import { STATUS_LABELS } from '../constants.ts';
import { $currentTeamId } from '../services/current-team.ts';
import { useDocumentTitle } from '../utils.ts';

function suggestedProjectName(repository: string): string {
    const repositoryName = repository.split('/').pop() ?? repository;
    const slug = repositoryName
        .toLowerCase()
        .replace(/[^a-z0-9]+/g, '-')
        .replace(/^-+|-+$/g, '')
        .slice(0, 63)
        .replace(/-+$/g, '');
    return slug || 'project';
}

export function DashboardPage() {
    const [projects, setProjects] = useState<Project[]>([]);
    const [teams, setTeams] = useState<Team[]>([]);
    const [repositories, setRepositories] = useState<string[]>([]);
    const [branches, setBranches] = useState<string[]>([]);
    const [loading, setLoading] = useState(true);
    const [loadError, setLoadError] = useState('');
    const [, navigate] = useLocation();
    const currentTeamId = $currentTeamId.value;
    const [teamGithubStatus, setTeamGithubStatus] = useState<TeamGithubStatusResponse | null>(null);
    useDocumentTitle('Home');

    const [creating, setCreating] = useState(false);
    const [form, setForm] = useState({ name: '', github_repo: '', github_branch: '', base_dir: '', team_id: '' });
    const [createError, setCreateError] = useState('');

    useEffect(() => {
        (async () => {
            try {
                const [projectData, teamData] = await Promise.all([
                    authFetch(`${API_URL}/projects`).then((r) => jsonOrThrow<ProjectIndexResponse>(r)),
                    authFetch(`${API_URL}/teams`).then((r) => jsonOrThrow<TeamIndexResponse>(r)),
                ]);
                setProjects(projectData.data);
                setTeams(teamData.data);
                setForm((prev) => ({ ...prev, team_id: currentTeamId ?? '' }));
            } catch {
                setLoadError('Failed to load projects.');
            } finally {
                setLoading(false);
            }
        })();
    }, []);

    useEffect(() => {
        setForm((prev) => ({ ...prev, team_id: currentTeamId ?? '', github_repo: '', github_branch: '' }));
    }, [currentTeamId]);

    useEffect(() => {
        if (!form.team_id) {
            setRepositories([]);
            setBranches([]);
            setTeamGithubStatus(null);
            setForm((prev) => ({ ...prev, github_repo: '', github_branch: '' }));
            return;
        }

        let ignore = false;
        loadTeamGithubRepositories(form.team_id).then(({ repositories, status }) => {
            if (ignore) return;
            setTeamGithubStatus(status);
            setRepositories(repositories);
            setForm((prev) => ({
                ...prev,
                github_repo: repositories.includes(prev.github_repo) ? prev.github_repo : '',
                github_branch: repositories.includes(prev.github_repo) ? prev.github_branch : '',
            }));
        });
        return () => {
            ignore = true;
        };
    }, [form.team_id]);

    useEffect(() => {
        if (!form.team_id || !form.github_repo) {
            setBranches([]);
            setForm((prev) => ({ ...prev, github_branch: prev.github_repo ? prev.github_branch : '' }));
            return;
        }

        let ignore = false;
        loadTeamGithubBranches(form.team_id, form.github_repo).then((nextBranches) => {
            if (ignore) return;
            setBranches(nextBranches);
            setForm((prev) => ({
                ...prev,
                github_branch: nextBranches.includes(prev.github_branch) ? prev.github_branch : (nextBranches[0] ?? ''),
            }));
        });
        return () => {
            ignore = true;
        };
    }, [form.github_repo, form.team_id]);

    async function handleCreate(e: Event) {
        e.preventDefault();
        setCreateError('');
        if (!repositories.includes(form.github_repo)) {
            setCreateError('Select a repository from the connected GitHub account.');
            return;
        }
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
                        <h1>Projects</h1>
                        <p class="page-subtitle">Deploy and manage the projects in your selected team.</p>
                    </div>
                    <span class="spacer" />
                    <Button class="is-small" onClick={() => setCreating(true)} disabled={!currentTeamId}>
                        <PlusIcon class="is-sm" />
                        New Project
                    </Button>
                </div>

                {creating && (
                    <Dialog title="New Project" onClose={() => setCreating(false)}>
                        {createError && <div class="notification is-danger">{createError}</div>}
                        <form onSubmit={handleCreate}>
                            <FormField
                                id="project-repo"
                                label="GitHub Repo"
                                help={
                                    form.team_id && teamGithubStatus?.connected === false
                                        ? 'Connect GitHub for the selected team on the Teams page before creating a project.'
                                        : undefined
                                }
                            >
                                <RepositoryInput
                                    id="project-repo"
                                    repositories={repositories}
                                    value={form.github_repo}
                                    onChange={(github_repo) => {
                                        setForm({
                                            ...form,
                                            github_repo,
                                            name: suggestedProjectName(github_repo),
                                        });
                                    }}
                                    disabled={!teamGithubStatus?.connected}
                                    placeholder={
                                        !form.team_id
                                            ? 'Choose a team in the navigation'
                                            : !teamGithubStatus?.connected
                                              ? 'Connect GitHub on the Teams page'
                                              : 'Type to find a repository'
                                    }
                                />
                            </FormField>
                            <FormField id="project-name" label="Name (subdomain slug)">
                                <FormInput
                                    id="project-name"
                                    value={form.name}
                                    onInput={(e) => setForm({ ...form, name: (e.target as HTMLInputElement).value })}
                                    placeholder="my-app"
                                    required
                                />
                            </FormField>
                            <FormField id="project-branch" label="Branch">
                                <FormSelect
                                    id="project-branch"
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
                            <FormField id="project-base-dir" label="Base Dir (optional, path within repo)">
                                <FormInput
                                    id="project-base-dir"
                                    value={form.base_dir}
                                    onInput={(e) =>
                                        setForm({ ...form, base_dir: (e.target as HTMLInputElement).value })
                                    }
                                    placeholder="apps/my-app"
                                />
                            </FormField>
                            <div class="buttons">
                                <Button type="submit">Create</Button>
                                <SecondaryButton type="button" onClick={() => setCreating(false)}>
                                    Cancel
                                </SecondaryButton>
                            </div>
                        </form>
                    </Dialog>
                )}

                {loadError && <div class="notification is-danger">{loadError}</div>}
                {loading && <div class="loading">Loading projects…</div>}
                {!loading && !loadError && !currentTeamId && (
                    <EmptyState icon={<RocketIcon />} message="No teams available yet." />
                )}
                {!loading &&
                    teams
                        .filter((team) => team.id === currentTeamId)
                        .map((team) => {
                            const teamProjects = projectsByTeam.get(team.id) ?? [];

                            return (
                                <Card key={team.id}>
                                    <div class="card-header">
                                        <div>
                                            <h2>{team.name}</h2>
                                            <p class="card-meta">
                                                {teamProjects.length} project{teamProjects.length === 1 ? '' : 's'}
                                            </p>
                                        </div>
                                    </div>

                                    {teamProjects.length === 0 ? (
                                        <EmptyState icon={<RocketIcon />} message="No projects yet." />
                                    ) : (
                                        <div class="project-list">
                                            {teamProjects.map((project) => (
                                                <div
                                                    key={project.id}
                                                    class="project-list-item"
                                                    role="link"
                                                    tabindex={0}
                                                    onClick={() => navigate(`/projects/${project.id}`)}
                                                    onKeyDown={(event) => {
                                                        if (event.key === 'Enter' || event.key === ' ') {
                                                            event.preventDefault();
                                                            navigate(`/projects/${project.id}`);
                                                        }
                                                    }}
                                                >
                                                    <div>
                                                        <strong>{project.name}</strong>
                                                        <div class="card-meta">
                                                            <a
                                                                href={`https://github.com/${project.githubRepo}`}
                                                                target="_blank"
                                                                rel="noreferrer"
                                                                onClick={(event) => event.stopPropagation()}
                                                            >
                                                                {project.githubRepo}
                                                            </a>{' '}
                                                            ({project.githubBranch})
                                                            {project.baseDir ? ` / ${project.baseDir}` : ''}
                                                        </div>
                                                    </div>
                                                    <span class={`tag is-${project.status}`}>
                                                        {STATUS_LABELS[project.status]}
                                                    </span>
                                                </div>
                                            ))}
                                        </div>
                                    )}
                                </Card>
                            );
                        })}
            </div>
        </AppLayout>
    );
}
