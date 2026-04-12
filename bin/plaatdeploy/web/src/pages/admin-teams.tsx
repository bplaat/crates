/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import { useEffect, useMemo, useState } from 'preact/hooks';
import { useLocation } from 'wouter-preact';
import { AdminLayout } from '../components/admin-layout.tsx';
import { ConfirmDialog, Dialog } from '../components/dialog.tsx';
import type { Project, ProjectIndexResponse, Team, TeamIndexResponse } from '../src-gen/api.ts';
import { $authUser, API_URL, authFetch } from '../services/auth.ts';

const STATUS_LABELS: Record<string, string> = {
    idle: 'Idle',
    building: 'Building',
    running: 'Running',
    failed: 'Failed',
};

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

    useEffect(() => {
        Promise.all([
            authFetch(`${API_URL}/teams`).then((response) => response.json()),
            authFetch(`${API_URL}/projects`).then((response) => response.json()),
        ]).then(([teamData, projectData]: [TeamIndexResponse, ProjectIndexResponse]) => {
            setTeams(teamData.data);
            setProjects(projectData.data);
            setLoading(false);
        });
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
                    <button
                        class="btn btn-primary btn-sm"
                        onClick={() => {
                            setError('');
                            setFormName('');
                            setCreating(true);
                        }}
                    >
                        + New Team
                    </button>
                </div>

                {error && <div class="alert alert-error">{error}</div>}
                {loading && <div class="loading">Loading teams...</div>}

                {!loading &&
                    teams.map((team) => {
                        const teamProjects = projectsByTeam.get(team.id) ?? [];
                        return (
                            <div key={team.id} class="card">
                                <div class="page-header" style="margin-bottom: 12px;">
                                    <h2 style="margin: 0;">{team.name}</h2>
                                    <span class="badge badge-idle">{teamProjects.length} projects</span>
                                    <span class="spacer" />
                                    <div class="section-actions">
                                        <button
                                            class="btn btn-secondary btn-sm"
                                            onClick={() => {
                                                setError('');
                                                setFormName(team.name);
                                                setEditingTeam(team);
                                            }}
                                        >
                                            Edit
                                        </button>
                                        <button
                                            class="btn btn-danger btn-sm"
                                            onClick={() => {
                                                setError('');
                                                setDeleteTeam(team);
                                            }}
                                        >
                                            Delete
                                        </button>
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
                                                <span>
                                                    <strong>{project.name}</strong> - {project.githubRepo}
                                                </span>
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

                {creating && (
                    <Dialog title="New Team" onClose={() => setCreating(false)}>
                        <form onSubmit={handleCreate}>
                            <div class="form-group">
                                <label>Name</label>
                                <input
                                    value={formName}
                                    onInput={(event) => setFormName((event.target as HTMLInputElement).value)}
                                    required
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

                {editingTeam && (
                    <Dialog title="Edit Team" onClose={() => setEditingTeam(null)}>
                        <form onSubmit={handleUpdate}>
                            <div class="form-group">
                                <label>Name</label>
                                <input
                                    value={formName}
                                    onInput={(event) => setFormName((event.target as HTMLInputElement).value)}
                                    required
                                />
                            </div>
                            <div class="dialog-actions">
                                <button class="btn btn-primary" type="submit">
                                    Save
                                </button>
                                <button class="btn btn-secondary" type="button" onClick={() => setEditingTeam(null)}>
                                    Cancel
                                </button>
                            </div>
                        </form>
                    </Dialog>
                )}

                {deleteTeam && (
                    <ConfirmDialog
                        title="Delete Team"
                        message={`Delete team "${deleteTeam.name}"?`}
                        confirmLabel="Delete"
                        onConfirm={handleDelete}
                        onClose={() => setDeleteTeam(null)}
                    />
                )}
            </div>
        </AdminLayout>
    );
}
