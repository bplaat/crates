/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import { useEffect, useMemo, useState } from 'preact/hooks';
import { AppLayout } from '../components/app-layout.tsx';
import { ConfirmDialog, Dialog } from '../components/dialog.tsx';
import type {
    GithubSetupStartResponse,
    Team,
    TeamGithubStatusResponse,
    TeamIndexResponse,
    TeamShowResponse,
    TeamUser,
} from '../src-gen/api.ts';
import { $authUser, API_URL, authFetch } from '../services/auth.ts';
import { capitalizeLabel } from '../utils.ts';

export function TeamsPage() {
    const authUser = $authUser.value;
    const [teams, setTeams] = useState<Team[]>([]);
    const [selectedTeamId, setSelectedTeamId] = useState<string>('');
    const [selectedTeam, setSelectedTeam] = useState<Team | null>(null);
    const [members, setMembers] = useState<TeamUser[]>([]);
    const [loading, setLoading] = useState(true);
    const [creating, setCreating] = useState(false);
    const [editing, setEditing] = useState(false);
    const [addingMember, setAddingMember] = useState(false);
    const [deletingTeam, setDeletingTeam] = useState(false);
    const [memberToDelete, setMemberToDelete] = useState<TeamUser | null>(null);
    const [createError, setCreateError] = useState('');
    const [teamForm, setTeamForm] = useState({ name: '' });
    const [editForm, setEditForm] = useState({ name: '' });
    const [memberForm, setMemberForm] = useState({ email: '', role: 'member' });
    const [memberError, setMemberError] = useState('');
    const [githubStatus, setGithubStatus] = useState<TeamGithubStatusResponse | null>(null);
    const [githubInstallationId, setGithubInstallationId] = useState('');

    async function loadTeams(preferredTeamId?: string) {
        const res = await authFetch(`${API_URL}/teams`);
        const data: TeamIndexResponse = await res.json();
        setTeams(data.data);
        const nextTeamId = preferredTeamId ?? data.data[0]?.id ?? '';
        setSelectedTeamId(nextTeamId);
        setLoading(false);
    }

    async function loadTeam(teamId: string) {
        if (!teamId) {
            setSelectedTeam(null);
            setMembers([]);
            setGithubStatus(null);
            setGithubInstallationId('');
            return;
        }
        const [teamRes, githubRes] = await Promise.all([
            authFetch(`${API_URL}/teams/${teamId}`),
            authFetch(`${API_URL}/teams/${teamId}/github`),
        ]);
        if (!teamRes.ok) return;
        const data: TeamShowResponse = await teamRes.json();
        setSelectedTeam(data.team);
        setMembers(data.members);
        setEditForm({ name: data.team.name });
        if (githubRes.ok) {
            const githubData: TeamGithubStatusResponse = await githubRes.json();
            setGithubStatus(githubData);
            setGithubInstallationId(githubData.connection?.installationId?.toString() ?? '');
        } else {
            setGithubStatus(null);
            setGithubInstallationId('');
        }
    }

    useEffect(() => {
        loadTeams();
    }, []);

    useEffect(() => {
        loadTeam(selectedTeamId);
    }, [selectedTeamId]);

    const canManageSelected = useMemo(() => {
        if (!selectedTeam || !authUser) return false;
        if (authUser.role === 'admin') return true;
        return members.some((member) => member.userId === authUser.id && member.role === 'owner');
    }, [authUser, members, selectedTeam]);

    async function handleCreate(e: Event) {
        e.preventDefault();
        setCreateError('');
        const res = await authFetch(`${API_URL}/teams`, {
            method: 'POST',
            body: new URLSearchParams(teamForm),
        });
        if (!res.ok) {
            setCreateError('Failed to create team.');
            return;
        }
        const team: Team = await res.json();
        setCreating(false);
        setTeamForm({ name: '' });
        await loadTeams(team.id);
    }

    async function handleUpdate(e: Event) {
        e.preventDefault();
        if (!selectedTeam) return;
        const res = await authFetch(`${API_URL}/teams/${selectedTeam.id}`, {
            method: 'PUT',
            body: new URLSearchParams(editForm),
        });
        if (!res.ok) return;
        const updated: Team = await res.json();
        setSelectedTeam(updated);
        setTeams((prev) => prev.map((team) => (team.id === updated.id ? updated : team)));
        setEditing(false);
    }

    async function handleDeleteTeam() {
        if (!selectedTeam) return;
        const res = await authFetch(`${API_URL}/teams/${selectedTeam.id}`, { method: 'DELETE' });
        if (!res.ok) return;
        const nextTeamId = teams.find((team) => team.id !== selectedTeam.id)?.id;
        setTeams((prev) => prev.filter((team) => team.id !== selectedTeam.id));
        setSelectedTeamId(nextTeamId ?? '');
        setDeletingTeam(false);
    }

    async function handleMemberCreate(e: Event) {
        e.preventDefault();
        if (!selectedTeam) return;
        setMemberError('');
        const res = await authFetch(`${API_URL}/teams/${selectedTeam.id}/members`, {
            method: 'POST',
            body: new URLSearchParams(memberForm),
        });
        if (!res.ok) {
            setMemberError('Failed to add member.');
            return;
        }
        const member: TeamUser = await res.json();
        setMembers((prev) =>
            [...prev, member].sort((a, b) =>
                `${a.firstName} ${a.lastName}`.localeCompare(`${b.firstName} ${b.lastName}`),
            ),
        );
        setMemberForm({ email: '', role: 'member' });
        setAddingMember(false);
    }

    async function handleMemberRoleChange(userId: string, role: string) {
        if (!selectedTeam) return;
        const res = await authFetch(`${API_URL}/teams/${selectedTeam.id}/members/${userId}`, {
            method: 'PUT',
            body: new URLSearchParams({ role }),
        });
        if (!res.ok) return;
        const member: TeamUser = await res.json();
        setMembers((prev) => prev.map((item) => (item.userId === userId ? member : item)));
    }

    async function handleMemberDelete() {
        if (!selectedTeam || !memberToDelete) return;
        const res = await authFetch(`${API_URL}/teams/${selectedTeam.id}/members/${memberToDelete.userId}`, {
            method: 'DELETE',
        });
        if (!res.ok) return;
        setMembers((prev) => prev.filter((member) => member.userId !== memberToDelete.userId));
        setMemberToDelete(null);
    }

    async function handleGithubSetup() {
        if (!selectedTeam) return;
        const res = await authFetch(`${API_URL}/teams/${selectedTeam.id}/github/setup-start`);
        if (!res.ok) return;
        const data: GithubSetupStartResponse = await res.json();
        const form = document.createElement('form');
        form.method = 'post';
        form.action = `https://github.com/settings/apps/new?state=${encodeURIComponent(data.state)}`;
        const input = document.createElement('input');
        input.type = 'hidden';
        input.name = 'manifest';
        input.value = JSON.stringify(data.manifest);
        form.appendChild(input);
        document.body.appendChild(form);
        form.submit();
    }

    async function handleGithubConnect(e: Event) {
        e.preventDefault();
        if (!selectedTeam || !githubInstallationId) return;
        const res = await authFetch(`${API_URL}/teams/${selectedTeam.id}/github`, {
            method: 'PUT',
            body: new URLSearchParams({ installation_id: githubInstallationId }),
        });
        if (!res.ok) return;
        const data: TeamGithubStatusResponse = await res.json();
        setGithubStatus(data);
        setGithubInstallationId(data.connection?.installationId?.toString() ?? githubInstallationId);
    }

    async function handleGithubDisconnect() {
        if (!selectedTeam) return;
        const res = await authFetch(`${API_URL}/teams/${selectedTeam.id}/github`, { method: 'DELETE' });
        if (!res.ok) return;
        await loadTeam(selectedTeam.id);
    }

    return (
        <AppLayout>
            <div class="page">
                <div class="page-header">
                    <h1>Teams</h1>
                    <button class="btn btn-primary btn-sm" onClick={() => setCreating(true)}>
                        + New Team
                    </button>
                </div>

                {creating && (
                    <Dialog title="New Team" onClose={() => setCreating(false)}>
                        {createError && <div class="alert alert-error">{createError}</div>}
                        <form onSubmit={handleCreate}>
                            <div class="form-group">
                                <label>Name</label>
                                <input
                                    value={teamForm.name}
                                    onInput={(e) => setTeamForm({ name: (e.target as HTMLInputElement).value })}
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

                {editing && selectedTeam && (
                    <Dialog title="Edit Team" onClose={() => setEditing(false)}>
                        <form onSubmit={handleUpdate}>
                            <div class="form-group">
                                <label>Name</label>
                                <input
                                    value={editForm.name}
                                    onInput={(e) => setEditForm({ name: (e.target as HTMLInputElement).value })}
                                    required
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

                {addingMember && selectedTeam && (
                    <Dialog title="Add Member" onClose={() => setAddingMember(false)}>
                        {memberError && <div class="alert alert-error">{memberError}</div>}
                        <form onSubmit={handleMemberCreate}>
                            <div class="form-group">
                                <label>User Email</label>
                                <input
                                    type="email"
                                    value={memberForm.email}
                                    onInput={(e) =>
                                        setMemberForm({
                                            ...memberForm,
                                            email: (e.target as HTMLInputElement).value,
                                        })
                                    }
                                    required
                                />
                            </div>
                            <div class="form-group">
                                <label>Role</label>
                                <select
                                    value={memberForm.role}
                                    onChange={(e) =>
                                        setMemberForm({
                                            ...memberForm,
                                            role: (e.target as HTMLSelectElement).value,
                                        })
                                    }
                                >
                                    <option value="member">Member</option>
                                    <option value="owner">Owner</option>
                                </select>
                            </div>
                            <div class="dialog-actions">
                                <button class="btn btn-primary" type="submit">
                                    Add Member
                                </button>
                                <button class="btn btn-secondary" type="button" onClick={() => setAddingMember(false)}>
                                    Cancel
                                </button>
                            </div>
                        </form>
                    </Dialog>
                )}

                {deletingTeam && selectedTeam && (
                    <ConfirmDialog
                        title="Delete Team"
                        message={`Delete team "${selectedTeam.name}"?`}
                        confirmLabel="Delete"
                        onConfirm={handleDeleteTeam}
                        onClose={() => setDeletingTeam(false)}
                    />
                )}

                {memberToDelete && (
                    <ConfirmDialog
                        title="Remove Member"
                        message={`Remove ${memberToDelete.firstName} ${memberToDelete.lastName} from this team?`}
                        confirmLabel="Remove"
                        onConfirm={handleMemberDelete}
                        onClose={() => setMemberToDelete(null)}
                    />
                )}

                {loading && <div class="loading">Loading teams...</div>}
                {!loading && teams.length === 0 && <div class="empty">No teams yet.</div>}
                {!loading && teams.length > 0 && (
                    <div class="two-column-grid">
                        <div class="card">
                            <h2>All Teams</h2>
                            <div class="selector-list">
                                {teams.map((team) => (
                                    <button
                                        key={team.id}
                                        class={`btn btn-secondary selector-item ${selectedTeamId === team.id ? 'active' : ''}`}
                                        onClick={() => setSelectedTeamId(team.id)}
                                    >
                                        <span>{team.name}</span>
                                    </button>
                                ))}
                            </div>
                        </div>

                        {selectedTeam && (
                            <div class="stack">
                                <div class="card">
                                    <div class="page-header" style="margin-bottom: 12px;">
                                        <h2 style="margin: 0;">{selectedTeam.name}</h2>
                                    </div>
                                    <p class="card-meta" style="margin-bottom: 16px;">
                                        Manage team details, members, and team-scoped GitHub access here.
                                    </p>
                                    <div class="card-actions">
                                        <button
                                            class="btn btn-primary"
                                            type="button"
                                            disabled={!canManageSelected}
                                            onClick={() => setEditing(true)}
                                        >
                                            Edit
                                        </button>
                                        <button
                                            class="btn btn-danger"
                                            type="button"
                                            disabled={!canManageSelected}
                                            onClick={() => setDeletingTeam(true)}
                                        >
                                            Delete
                                        </button>
                                    </div>
                                </div>

                                <div class="card">
                                    <div class="page-header" style="margin-bottom: 12px;">
                                        <h2 style="margin: 0;">Members</h2>
                                        {canManageSelected && (
                                            <button
                                                class="btn btn-primary btn-sm"
                                                type="button"
                                                onClick={() => setAddingMember(true)}
                                            >
                                                + Add Member
                                            </button>
                                        )}
                                    </div>
                                    <table>
                                        <thead>
                                            <tr>
                                                <th>Name</th>
                                                <th>Email</th>
                                                <th>Role</th>
                                                <th>Added</th>
                                                <th></th>
                                            </tr>
                                        </thead>
                                        <tbody>
                                            {members.map((member) => (
                                                <tr key={member.userId}>
                                                    <td>{`${member.firstName} ${member.lastName}`.trim()}</td>
                                                    <td>{member.email}</td>
                                                    <td>
                                                        {canManageSelected ? (
                                                            <select
                                                                value={member.role}
                                                                onChange={(e) =>
                                                                    handleMemberRoleChange(
                                                                        member.userId,
                                                                        (e.target as HTMLSelectElement).value,
                                                                    )
                                                                }
                                                            >
                                                                <option value="member">Member</option>
                                                                <option value="owner">Owner</option>
                                                            </select>
                                                        ) : (
                                                            <span class="role-badge">
                                                                {capitalizeLabel(member.role)}
                                                            </span>
                                                        )}
                                                    </td>
                                                    <td class="muted-text">
                                                        {new Date(member.createdAt).toLocaleDateString()}
                                                    </td>
                                                    <td>
                                                        {canManageSelected && authUser?.id !== member.userId && (
                                                            <button
                                                                class="btn btn-danger btn-sm"
                                                                type="button"
                                                                onClick={() => setMemberToDelete(member)}
                                                            >
                                                                Remove
                                                            </button>
                                                        )}
                                                    </td>
                                                </tr>
                                            ))}
                                        </tbody>
                                    </table>
                                </div>

                                <div class="card">
                                    <h2>GitHub</h2>
                                    {!githubStatus?.appConfigured ? (
                                        <div class="page-header" style="justify-content: space-between;">
                                            <div>
                                                <strong>GitHub App not configured</strong>
                                                <p class="card-meta">
                                                    Configure the GitHub App for this team before connecting
                                                    repositories.
                                                </p>
                                            </div>
                                            {canManageSelected && (
                                                <button class="btn btn-primary btn-sm" onClick={handleGithubSetup}>
                                                    Setup GitHub App
                                                </button>
                                            )}
                                        </div>
                                    ) : (
                                        <div class="stack">
                                            {githubStatus.connection ? (
                                                <div>
                                                    <strong>Connected to {githubStatus.connection.accountLogin}</strong>
                                                    <p class="card-meta">
                                                        Installation #{githubStatus.connection.installationId}
                                                        {githubStatus.inheritedConnection
                                                            ? ' inherited from the default installation.'
                                                            : '.'}
                                                    </p>
                                                </div>
                                            ) : (
                                                <div class="empty" style="margin: 0;">
                                                    No GitHub installation connected for this team yet.
                                                </div>
                                            )}

                                            {githubStatus.installUrl && (
                                                <div>
                                                    <a href={githubStatus.installUrl} target="_blank" rel="noreferrer">
                                                        Install GitHub App
                                                    </a>
                                                </div>
                                            )}

                                            {canManageSelected && githubStatus.installations.length > 0 && (
                                                <form onSubmit={handleGithubConnect}>
                                                    <div class="form-group">
                                                        <label>Installation</label>
                                                        <select
                                                            value={githubInstallationId}
                                                            onChange={(e) =>
                                                                setGithubInstallationId(
                                                                    (e.target as HTMLSelectElement).value,
                                                                )
                                                            }
                                                            required
                                                        >
                                                            <option value="" disabled>
                                                                Select an installation
                                                            </option>
                                                            {githubStatus.installations.map((installation) => (
                                                                <option key={installation.id} value={installation.id}>
                                                                    {installation.accountLogin}
                                                                    {installation.accountType
                                                                        ? ` (${installation.accountType})`
                                                                        : ''}
                                                                </option>
                                                            ))}
                                                        </select>
                                                    </div>
                                                    <div class="card-actions">
                                                        <button
                                                            class="btn btn-primary"
                                                            type="submit"
                                                            disabled={!githubInstallationId}
                                                        >
                                                            {githubStatus.connection
                                                                ? 'Switch Installation'
                                                                : 'Connect Installation'}
                                                        </button>
                                                        {githubStatus.connection &&
                                                            !githubStatus.inheritedConnection && (
                                                                <button
                                                                    class="btn btn-danger"
                                                                    type="button"
                                                                    onClick={handleGithubDisconnect}
                                                                >
                                                                    Disconnect
                                                                </button>
                                                            )}
                                                    </div>
                                                </form>
                                            )}

                                            {canManageSelected &&
                                                githubStatus.installations.length === 0 &&
                                                !githubStatus.installUrl && (
                                                    <div class="empty" style="margin: 0;">
                                                        Install the GitHub App on an account first, then reconnect this
                                                        page.
                                                    </div>
                                                )}
                                        </div>
                                    )}
                                </div>
                            </div>
                        )}
                    </div>
                )}
            </div>
        </AppLayout>
    );
}
