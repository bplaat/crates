/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import { useEffect, useMemo, useState } from 'preact/hooks';
import { useLocation } from 'wouter-preact';
import { AdminLayout } from '../components/admin-layout.tsx';
import { ConfirmDialog, Dialog } from '../components/dialog.tsx';
import { Button, DangerButton, SecondaryButton } from '../components/button.tsx';
import { Card } from '../components/card.tsx';
import { EmptyState } from '../components/empty-state.tsx';
import { FormField } from '../components/form.tsx';
import { FormInput } from '../components/input.tsx';
import { DeleteIcon, PencilIcon, PlusIcon, RocketIcon } from '../components/icons.tsx';
import type { Project, ProjectIndexResponse, Team, TeamIndexResponse } from '../src-gen/api.ts';
import { $authUser, API_URL, authFetch } from '../services/auth.ts';
import { jsonOrThrow } from '../services/api.ts';
import { STATUS_LABELS } from '../constants.ts';
import { useDocumentTitle } from '../utils.ts';

export function AdminTeamsPage() {
    const authUser = $authUser.value;
    const [teams, setTeams] = useState<Team[]>([]);
    const [projects, setProjects] = useState<Project[]>([]);
    const [loading, setLoading] = useState(true);
    const [creating, setCreating] = useState(false);
    const [editingTeam, setEditingTeam] = useState<Team | null>(null);
    const [deleteTeam, setDeleteTeam] = useState<Team | null>(null);
    const [formName, setFormName] = useState('');
    const [error, setError] = useState('');
    const [, navigate] = useLocation();
    useDocumentTitle('Teams');

    useEffect(() => {
        (async () => {
            try {
                const [teamData, projectData] = await Promise.all([
                    authFetch(`${API_URL}/teams`).then((response) => jsonOrThrow<TeamIndexResponse>(response)),
                    authFetch(`${API_URL}/projects`).then((response) => jsonOrThrow<ProjectIndexResponse>(response)),
                ]);
                setTeams(teamData.data);
                setProjects(projectData.data);
            } catch {
                setError('Failed to load teams.');
            } finally {
                setLoading(false);
            }
        })();
    }, []);

    const projectsByTeam = useMemo(() => {
        return new Map(teams.map((team) => [team.id, projects.filter((project) => project.teamId === team.id)]));
    }, [projects, teams]);

    if (authUser?.role !== 'admin') {
        return (
            <AdminLayout>
                <div class="page">
                    <div class="empty">Access denied.</div>
                </div>
            </AdminLayout>
        );
    }

    async function handleCreate(event: Event) {
        event.preventDefault();
        setError('');
        const response = await authFetch(`${API_URL}/teams`, {
            method: 'POST',
            body: new URLSearchParams({ name: formName }),
        });
        if (!response.ok) {
            setError('Failed to create team.');
            return;
        }
        const team = (await response.json()) as Team;
        setTeams((current) => [team, ...current]);
        setCreating(false);
        setFormName('');
    }

    async function handleUpdate(event: Event) {
        event.preventDefault();
        if (!editingTeam) return;
        setError('');
        const response = await authFetch(`${API_URL}/teams/${editingTeam.id}`, {
            method: 'PUT',
            body: new URLSearchParams({ name: formName }),
        });
        if (!response.ok) {
            setError('Failed to update team.');
            return;
        }
        const updatedTeam = (await response.json()) as Team;
        setTeams((current) => current.map((team) => (team.id === updatedTeam.id ? updatedTeam : team)));
        setEditingTeam(null);
        setFormName('');
    }

    async function handleDelete() {
        if (!deleteTeam) return;
        setError('');
        const response = await authFetch(`${API_URL}/teams/${deleteTeam.id}`, {
            method: 'DELETE',
        });
        if (!response.ok) {
            setError('Failed to delete team. Delete or move its projects first.');
            setDeleteTeam(null);
            return;
        }
        setTeams((current) => current.filter((team) => team.id !== deleteTeam.id));
        setProjects((current) => current.filter((project) => project.teamId !== deleteTeam.id));
        setDeleteTeam(null);
    }

    return (
        <AdminLayout>
            <div class="page">
                <div class="page-header">
                    <h1>Teams</h1>
                    <span class="spacer" />
                    <Button
                        class="is-small"
                        onClick={() => {
                            setError('');
                            setFormName('');
                            setCreating(true);
                        }}
                    >
                        <PlusIcon class="is-sm" />
                        New Team
                    </Button>
                </div>

                {error && <div class="notification is-danger">{error}</div>}
                {loading && <div class="loading">Loading teams…</div>}

                {!loading &&
                    teams.map((team) => {
                        const teamProjects = projectsByTeam.get(team.id) ?? [];
                        return (
                            <Card key={team.id}>
                                <div class="card-header">
                                    <h2>{team.name}</h2>
                                    <span class="tag is-idle">{teamProjects.length} projects</span>
                                    <span class="spacer" />
                                    <div class="buttons">
                                        <SecondaryButton
                                            class="is-small"
                                            onClick={() => {
                                                setError('');
                                                setFormName(team.name);
                                                setEditingTeam(team);
                                            }}
                                        >
                                            <PencilIcon class="is-sm" />
                                            Edit
                                        </SecondaryButton>
                                        <DangerButton
                                            class="is-small"
                                            onClick={() => {
                                                setError('');
                                                setDeleteTeam(team);
                                            }}
                                        >
                                            <DeleteIcon class="is-sm" />
                                            Delete
                                        </DangerButton>
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
                                                <span>
                                                    <strong>{project.name}</strong> -{' '}
                                                    <a
                                                        href={`https://github.com/${project.githubRepo}`}
                                                        target="_blank"
                                                        rel="noreferrer"
                                                        onClick={(event) => event.stopPropagation()}
                                                    >
                                                        {project.githubRepo}
                                                    </a>
                                                </span>
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

                {creating && (
                    <Dialog title="New Team" onClose={() => setCreating(false)}>
                        <form onSubmit={handleCreate}>
                            <FormField id="new-team-name" label="Name">
                                <FormInput
                                    id="new-team-name"
                                    value={formName}
                                    onInput={(event) => setFormName((event.target as HTMLInputElement).value)}
                                    required
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

                {editingTeam && (
                    <Dialog title="Edit Team" onClose={() => setEditingTeam(null)}>
                        <form onSubmit={handleUpdate}>
                            <FormField id="edit-team-name" label="Name">
                                <FormInput
                                    id="edit-team-name"
                                    value={formName}
                                    onInput={(event) => setFormName((event.target as HTMLInputElement).value)}
                                    required
                                />
                            </FormField>
                            <div class="buttons">
                                <Button type="submit">Save</Button>
                                <SecondaryButton type="button" onClick={() => setEditingTeam(null)}>
                                    Cancel
                                </SecondaryButton>
                            </div>
                        </form>
                    </Dialog>
                )}

                {deleteTeam && (
                    <ConfirmDialog
                        title="Delete Team"
                        message={`Delete team "${deleteTeam.name}"? This also deletes its projects and deployments.`}
                        confirmLabel="Delete"
                        confirmationText={deleteTeam.name}
                        onConfirm={handleDelete}
                        onClose={() => setDeleteTeam(null)}
                    />
                )}
            </div>
        </AdminLayout>
    );
}
