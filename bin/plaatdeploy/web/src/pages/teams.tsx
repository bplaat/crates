/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import { useEffect, useMemo, useState } from 'preact/hooks';
import { AppLayout } from '../components/app-layout.tsx';
import { ConfirmDialog, Dialog } from '../components/dialog.tsx';
import { Button, DangerButton, SecondaryButton } from '../components/button.tsx';
import { Card } from '../components/card.tsx';
import { FormField } from '../components/form.tsx';
import { FormInput, FormSelect } from '../components/input.tsx';
import { DeleteIcon, GithubIcon, OpenInNewIcon, PencilIcon, PlusIcon } from '../components/icons.tsx';
import type { Team, TeamGithubStatusResponse, TeamIndexResponse, TeamShowResponse, TeamUser } from '../src-gen/api.ts';
import { $authUser, API_URL, authFetch } from '../services/auth.ts';
import { jsonOrThrow } from '../services/api.ts';
import { capitalizeLabel, useDocumentTitle } from '../utils.ts';
import { $currentTeamId, setCurrentTeamId } from '../services/current-team.ts';

export function TeamsPage() {
    const authUser = $authUser.value;
    const currentTeamId = $currentTeamId.value;
    const [teams, setTeams] = useState<Team[]>([]);
    const [selectedTeamId, setSelectedTeamId] = useState<string>('');
    const [selectedTeam, setSelectedTeam] = useState<Team | null>(null);
    const [members, setMembers] = useState<TeamUser[]>([]);
    const [loading, setLoading] = useState(true);
    const [loadError, setLoadError] = useState('');
    const [actionError, setActionError] = useState('');
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
    const [githubToken, setGithubToken] = useState('');
    useDocumentTitle('Teams');

    async function loadTeams(preferredTeamId?: string) {
        try {
            const data = await authFetch(`${API_URL}/teams`).then((res) => jsonOrThrow<TeamIndexResponse>(res));
            setTeams(data.data);
            const nextTeamId = preferredTeamId ?? currentTeamId ?? data.data[0]?.id ?? '';
            setSelectedTeamId(nextTeamId);
            setCurrentTeamId(nextTeamId || null);
        } catch {
            setLoadError('Failed to load teams.');
        } finally {
            setLoading(false);
        }
    }

    async function loadTeam(teamId: string, isCancelled: () => boolean = () => false) {
        if (!teamId) {
            if (isCancelled()) return;
            setSelectedTeam(null);
            setMembers([]);
            setGithubStatus(null);
            setGithubToken('');
            return;
        }
        const [teamRes, githubRes] = await Promise.all([
            authFetch(`${API_URL}/teams/${teamId}`),
            authFetch(`${API_URL}/teams/${teamId}/github`),
        ]);
        if (isCancelled() || !teamRes.ok) return;
        const data: TeamShowResponse = await teamRes.json();
        if (isCancelled()) return;
        setSelectedTeam(data.team);
        setMembers(data.members);
        setEditForm({ name: data.team.name });
        if (githubRes.ok) {
            const githubData: TeamGithubStatusResponse = await githubRes.json();
            if (isCancelled()) return;
            setGithubStatus(githubData);
            setGithubToken('');
        } else {
            setGithubStatus(null);
            setGithubToken('');
        }
    }

    useEffect(() => {
        loadTeams();
    }, []);

    useEffect(() => {
        if (currentTeamId) setSelectedTeamId(currentTeamId);
    }, [currentTeamId]);

    useEffect(() => {
        let ignore = false;
        loadTeam(selectedTeamId, () => ignore);
        return () => {
            ignore = true;
        };
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
        setActionError('');
        const res = await authFetch(`${API_URL}/teams/${selectedTeam.id}`, {
            method: 'PUT',
            body: new URLSearchParams(editForm),
        });
        if (!res.ok) {
            setActionError('Failed to update team.');
            return;
        }
        const updated: Team = await res.json();
        setSelectedTeam(updated);
        setTeams((prev) => prev.map((team) => (team.id === updated.id ? updated : team)));
        setEditing(false);
    }

    async function handleDeleteTeam() {
        if (!selectedTeam) return;
        setActionError('');
        const res = await authFetch(`${API_URL}/teams/${selectedTeam.id}`, { method: 'DELETE' });
        if (!res.ok) {
            setActionError('Failed to delete team. Delete or move its projects first.');
            setDeletingTeam(false);
            return;
        }
        const nextTeamId = teams.find((team) => team.id !== selectedTeam.id)?.id;
        setTeams((prev) => prev.filter((team) => team.id !== selectedTeam.id));
        setSelectedTeamId(nextTeamId ?? '');
        setCurrentTeamId(nextTeamId ?? null);
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
        setActionError('');
        const res = await authFetch(`${API_URL}/teams/${selectedTeam.id}/members/${userId}`, {
            method: 'PUT',
            body: new URLSearchParams({ role }),
        });
        if (!res.ok) {
            setActionError('Failed to update member role.');
            return;
        }
        const member: TeamUser = await res.json();
        setMembers((prev) => prev.map((item) => (item.userId === userId ? member : item)));
    }

    async function handleMemberDelete() {
        if (!selectedTeam || !memberToDelete) return;
        setActionError('');
        const res = await authFetch(`${API_URL}/teams/${selectedTeam.id}/members/${memberToDelete.userId}`, {
            method: 'DELETE',
        });
        if (!res.ok) {
            setActionError('Failed to remove member.');
            setMemberToDelete(null);
            return;
        }
        setMembers((prev) => prev.filter((member) => member.userId !== memberToDelete.userId));
        setMemberToDelete(null);
    }

    async function handleGithubConnect(e: Event) {
        e.preventDefault();
        if (!selectedTeam || !githubToken) return;
        setActionError('');
        const res = await authFetch(`${API_URL}/teams/${selectedTeam.id}/github`, {
            method: 'PUT',
            body: new URLSearchParams({ access_token: githubToken }),
        });
        if (!res.ok) {
            setActionError('GitHub could not validate this token. Check its permissions and try again.');
            return;
        }
        const data: TeamGithubStatusResponse = await res.json();
        setGithubStatus(data);
        setGithubToken('');
    }

    async function handleGithubDisconnect() {
        if (!selectedTeam) return;
        setActionError('');
        const res = await authFetch(`${API_URL}/teams/${selectedTeam.id}/github`, { method: 'DELETE' });
        if (!res.ok) {
            setActionError('Failed to disconnect GitHub.');
            return;
        }
        await loadTeam(selectedTeam.id);
    }

    return (
        <AppLayout>
            <div class="page">
                <div class="page-header">
                    <h1>Teams</h1>
                    <span class="spacer" />
                    <Button class="is-small" onClick={() => setCreating(true)}>
                        <PlusIcon class="is-sm" />
                        New Team
                    </Button>
                </div>

                {loadError && <div class="notification is-danger">{loadError}</div>}
                {actionError && <div class="notification is-danger">{actionError}</div>}

                {creating && (
                    <Dialog title="New Team" onClose={() => setCreating(false)}>
                        {createError && <div class="notification is-danger">{createError}</div>}
                        <form onSubmit={handleCreate}>
                            <FormField id="new-team-name" label="Name">
                                <FormInput
                                    id="new-team-name"
                                    value={teamForm.name}
                                    onInput={(e) => setTeamForm({ name: (e.target as HTMLInputElement).value })}
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

                {editing && selectedTeam && (
                    <Dialog title="Edit Team" onClose={() => setEditing(false)}>
                        <form onSubmit={handleUpdate}>
                            <FormField id="edit-team-name" label="Name">
                                <FormInput
                                    id="edit-team-name"
                                    value={editForm.name}
                                    onInput={(e) => setEditForm({ name: (e.target as HTMLInputElement).value })}
                                    required
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

                {addingMember && selectedTeam && (
                    <Dialog title="Add Member" onClose={() => setAddingMember(false)}>
                        {memberError && <div class="notification is-danger">{memberError}</div>}
                        <form onSubmit={handleMemberCreate}>
                            <FormField id="new-member-email" label="User Email">
                                <FormInput
                                    id="new-member-email"
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
                            </FormField>
                            <FormField id="new-member-role" label="Role">
                                <FormSelect
                                    id="new-member-role"
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
                                </FormSelect>
                            </FormField>
                            <div class="buttons">
                                <Button type="submit">Add Member</Button>
                                <SecondaryButton type="button" onClick={() => setAddingMember(false)}>
                                    Cancel
                                </SecondaryButton>
                            </div>
                        </form>
                    </Dialog>
                )}

                {deletingTeam && selectedTeam && (
                    <ConfirmDialog
                        title="Delete Team"
                        message={`Delete team "${selectedTeam.name}"? This also deletes its projects and deployments.`}
                        confirmLabel="Delete"
                        confirmationText={selectedTeam.name}
                        onConfirm={handleDeleteTeam}
                        onClose={() => setDeletingTeam(false)}
                    />
                )}

                {memberToDelete && (
                    <ConfirmDialog
                        title="Remove Member"
                        message={`Remove ${memberToDelete.firstName} ${memberToDelete.lastName} from this team?`}
                        confirmLabel="Remove"
                        confirmationText={memberToDelete.email}
                        onConfirm={handleMemberDelete}
                        onClose={() => setMemberToDelete(null)}
                    />
                )}

                {loading && <div class="loading">Loading teams…</div>}
                {!loading && teams.length === 0 && <div class="empty">No teams yet.</div>}
                {!loading && selectedTeam && (
                    <div class="stack">
                        <Card>
                            <div class="card-header">
                                <h2>{selectedTeam.name}</h2>
                            </div>
                            <p class="card-meta">Manage team details, members, and team-scoped GitHub access here.</p>
                            <div class="buttons" style="margin-top: 16px;">
                                <Button type="button" disabled={!canManageSelected} onClick={() => setEditing(true)}>
                                    <PencilIcon class="is-sm" />
                                    Edit
                                </Button>
                                <DangerButton
                                    type="button"
                                    disabled={!canManageSelected}
                                    onClick={() => setDeletingTeam(true)}
                                >
                                    <DeleteIcon class="is-sm" />
                                    Delete
                                </DangerButton>
                            </div>
                        </Card>

                        <Card>
                            <div class="card-header">
                                <h2>Members</h2>
                                <span class="spacer" />
                                {canManageSelected && (
                                    <Button class="is-small" type="button" onClick={() => setAddingMember(true)}>
                                        <PlusIcon class="is-sm" />
                                        Add Member
                                    </Button>
                                )}
                            </div>
                            <div class="table-wrap">
                                <table class="table">
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
                                                        <FormSelect
                                                            aria-label="Member role"
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
                                                        </FormSelect>
                                                    ) : (
                                                        <span class="role-badge">{capitalizeLabel(member.role)}</span>
                                                    )}
                                                </td>
                                                <td class="has-text-muted">
                                                    {new Date(member.createdAt).toLocaleDateString()}
                                                </td>
                                                <td>
                                                    {canManageSelected && authUser?.id !== member.userId && (
                                                        <DangerButton
                                                            class="is-small"
                                                            type="button"
                                                            onClick={() => setMemberToDelete(member)}
                                                        >
                                                            Remove
                                                        </DangerButton>
                                                    )}
                                                </td>
                                            </tr>
                                        ))}
                                    </tbody>
                                </table>
                            </div>
                        </Card>

                        <Card>
                            <div class="card-header">
                                <h2 class="icon-text">
                                    <GithubIcon class="is-sm" />
                                    GitHub
                                </h2>
                            </div>
                            <div class="stack">
                                {githubStatus?.connection ? (
                                    <div>
                                        <strong>Connected as {githubStatus.connection.accountLogin}</strong>
                                        <p class="card-meta">Fine-grained personal access token.</p>
                                    </div>
                                ) : (
                                    <p class="card-meta">
                                        Connect a fine-grained personal access token to select repositories and receive
                                        deployments.
                                    </p>
                                )}
                                {canManageSelected && (
                                    <>
                                        <details class="github-token-guide" open={!githubStatus?.connected}>
                                            <summary>Create a fine-grained token</summary>
                                            <ol>
                                                <li>
                                                    Open GitHub Settings, then Developer settings and Fine-grained
                                                    personal access tokens.
                                                </li>
                                                <li>
                                                    Create a token for the account or organization that owns the
                                                    repositories.
                                                </li>
                                                <li>
                                                    Choose <strong>Only select repositories</strong> and select the
                                                    repositories this team deploys.
                                                </li>
                                                <li>
                                                    Set these repository permissions: <strong>Metadata: Read</strong>,{' '}
                                                    <strong>Contents: Read</strong>, <strong>Actions: Read</strong>,{' '}
                                                    <strong>Deployments: Read and write</strong>, and{' '}
                                                    <strong>Webhooks: Read and write</strong>. The token owner also
                                                    needs repository admin access to create or delete webhooks.
                                                </li>
                                            </ol>
                                            <p class="card-meta">
                                                PlaatDeploy uses the token to clone code, wait for CI, publish
                                                deployment status, and create the repository webhook automatically when
                                                you add a project.
                                            </p>
                                            <a
                                                class="icon-text"
                                                href="https://github.com/settings/personal-access-tokens/new"
                                                target="_blank"
                                                rel="noreferrer"
                                            >
                                                Create token on GitHub
                                                <OpenInNewIcon class="is-sm" />
                                            </a>
                                        </details>
                                        <form onSubmit={handleGithubConnect}>
                                            <FormField id="github-token" label="Fine-grained personal access token">
                                                <FormInput
                                                    id="github-token"
                                                    type="password"
                                                    autocomplete="off"
                                                    placeholder="github_pat_…"
                                                    value={githubToken}
                                                    onInput={(e) =>
                                                        setGithubToken((e.target as HTMLInputElement).value)
                                                    }
                                                    required
                                                />
                                            </FormField>
                                            <div class="buttons">
                                                <Button type="submit" disabled={!githubToken}>
                                                    {githubStatus?.connected ? 'Replace Token' : 'Connect GitHub'}
                                                </Button>
                                                {githubStatus?.connected && (
                                                    <DangerButton type="button" onClick={handleGithubDisconnect}>
                                                        Disconnect
                                                    </DangerButton>
                                                )}
                                            </div>
                                        </form>
                                    </>
                                )}
                            </div>
                        </Card>
                    </div>
                )}
            </div>
        </AppLayout>
    );
}
