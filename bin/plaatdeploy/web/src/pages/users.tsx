/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import { useEffect, useState } from 'preact/hooks';
import { useLocation } from 'wouter-preact';
import { ConfirmDialog, Dialog } from '../components/dialog.tsx';
import { Button, DangerButton, SecondaryButton } from '../components/button.tsx';
import { Card } from '../components/card.tsx';
import { FormField } from '../components/form.tsx';
import { FormInput, FormSelect } from '../components/input.tsx';
import { DeleteIcon, PencilIcon, PlusIcon } from '../components/icons.tsx';
import type { User, UserIndexResponse } from '../src-gen/api.ts';
import { $authUser, API_URL, authFetch, logout } from '../services/auth.ts';
import { jsonOrThrow } from '../services/api.ts';
import { capitalizeLabel, useDocumentTitle } from '../utils.ts';

export function UsersPage() {
    const authUser = $authUser.value;
    const [, navigate] = useLocation();
    const [users, setUsers] = useState<User[]>([]);
    const [loading, setLoading] = useState(true);
    const [loadError, setLoadError] = useState('');
    const [creating, setCreating] = useState(false);
    const [form, setForm] = useState({
        first_name: '',
        last_name: '',
        email: '',
        password: '',
        role: 'normal',
    });
    const [createError, setCreateError] = useState('');
    const [editingUserId, setEditingUserId] = useState<string | null>(null);
    const [editForm, setEditForm] = useState({
        first_name: '',
        last_name: '',
        email: '',
        password: '',
        role: 'normal',
    });
    const [editError, setEditError] = useState('');
    const [deleteUser, setDeleteUser] = useState<User | null>(null);
    useDocumentTitle('Users');

    useEffect(() => {
        (async () => {
            try {
                const data = await authFetch(`${API_URL}/users`).then((r) => jsonOrThrow<UserIndexResponse>(r));
                setUsers(data.data);
            } catch {
                setLoadError('Failed to load users.');
            } finally {
                setLoading(false);
            }
        })();
    }, []);

    async function handleCreate(e: Event) {
        e.preventDefault();
        setCreateError('');
        const body = new URLSearchParams(form);
        const res = await authFetch(`${API_URL}/users`, { method: 'POST', body });
        if (!res.ok) {
            setCreateError('Failed to create user.');
            return;
        }
        const user: User = await res.json();
        setUsers((prev) => [user, ...prev]);
        setCreating(false);
        setForm({ first_name: '', last_name: '', email: '', password: '', role: 'normal' });
    }

    async function handleDelete() {
        if (!deleteUser) return;
        const res = await authFetch(`${API_URL}/users/${deleteUser.id}`, { method: 'DELETE' });
        if (!res.ok) return;

        const deletedOwnUser = deleteUser.id === authUser?.id;
        setUsers((prev) => prev.filter((u) => u.id !== deleteUser.id));
        setDeleteUser(null);

        if (deletedOwnUser) {
            await logout();
            navigate('/');
        }
    }

    function startEditing(user: User) {
        setEditingUserId(user.id);
        setEditError('');
        setEditForm({
            first_name: user.firstName,
            last_name: user.lastName,
            email: user.email,
            password: '',
            role: user.role,
        });
    }

    async function handleUpdate(e: Event) {
        e.preventDefault();
        if (!editingUserId) return;

        setEditError('');
        const body = new URLSearchParams(editForm);
        const res = await authFetch(`${API_URL}/users/${editingUserId}`, { method: 'PUT', body });
        if (!res.ok) {
            setEditError('Failed to update user.');
            return;
        }

        const updatedUser: User = await res.json();
        setUsers((prev) => prev.map((user) => (user.id === updatedUser.id ? updatedUser : user)));
        if (authUser?.id === updatedUser.id) {
            $authUser.value = updatedUser;
        }
        setEditingUserId(null);
    }

    if (authUser?.role !== 'admin') {
        return <div class="empty">Access denied.</div>;
    }

    return (
        <div class="page">
            <div class="page-header">
                <h1>Users</h1>
                <span class="spacer" />
                <Button class="is-small" onClick={() => setCreating(true)}>
                    <PlusIcon class="is-sm" />
                    New User
                </Button>
            </div>

            <Card>
                <p class="card-meta">
                    Creating a user also creates a default "{`First Last's Team`}" team for them automatically.
                </p>
            </Card>

            {creating && (
                <Dialog title="New User" onClose={() => setCreating(false)}>
                    {createError && <div class="notification is-danger">{createError}</div>}
                    <form onSubmit={handleCreate}>
                        <div class="field-row">
                            <FormField id="new-user-first-name" label="First Name">
                                <FormInput
                                    id="new-user-first-name"
                                    value={form.first_name}
                                    onInput={(e) =>
                                        setForm({ ...form, first_name: (e.target as HTMLInputElement).value })
                                    }
                                    required
                                />
                            </FormField>
                            <FormField id="new-user-last-name" label="Last Name">
                                <FormInput
                                    id="new-user-last-name"
                                    value={form.last_name}
                                    onInput={(e) =>
                                        setForm({ ...form, last_name: (e.target as HTMLInputElement).value })
                                    }
                                    required
                                />
                            </FormField>
                        </div>
                        <FormField id="new-user-email" label="Email">
                            <FormInput
                                id="new-user-email"
                                type="email"
                                value={form.email}
                                onInput={(e) => setForm({ ...form, email: (e.target as HTMLInputElement).value })}
                                required
                            />
                        </FormField>
                        <FormField id="new-user-password" label="Password">
                            <FormInput
                                id="new-user-password"
                                type="password"
                                value={form.password}
                                onInput={(e) => setForm({ ...form, password: (e.target as HTMLInputElement).value })}
                                required
                            />
                        </FormField>
                        <FormField id="new-user-role" label="Role">
                            <FormSelect
                                id="new-user-role"
                                value={form.role}
                                onChange={(e) => setForm({ ...form, role: (e.target as HTMLSelectElement).value })}
                            >
                                <option value="normal">Normal</option>
                                <option value="admin">Admin</option>
                            </FormSelect>
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

            {editingUserId && (
                <Dialog title="Edit User" onClose={() => setEditingUserId(null)}>
                    {editError && <div class="notification is-danger">{editError}</div>}
                    <form onSubmit={handleUpdate}>
                        <div class="field-row">
                            <FormField id="edit-user-first-name" label="First Name">
                                <FormInput
                                    id="edit-user-first-name"
                                    value={editForm.first_name}
                                    onInput={(e) =>
                                        setEditForm({ ...editForm, first_name: (e.target as HTMLInputElement).value })
                                    }
                                    required
                                />
                            </FormField>
                            <FormField id="edit-user-last-name" label="Last Name">
                                <FormInput
                                    id="edit-user-last-name"
                                    value={editForm.last_name}
                                    onInput={(e) =>
                                        setEditForm({ ...editForm, last_name: (e.target as HTMLInputElement).value })
                                    }
                                    required
                                />
                            </FormField>
                        </div>
                        <FormField id="edit-user-email" label="Email">
                            <FormInput
                                id="edit-user-email"
                                type="email"
                                value={editForm.email}
                                onInput={(e) =>
                                    setEditForm({ ...editForm, email: (e.target as HTMLInputElement).value })
                                }
                                required
                            />
                        </FormField>
                        <FormField id="edit-user-password" label="Password">
                            <FormInput
                                id="edit-user-password"
                                type="password"
                                value={editForm.password}
                                onInput={(e) =>
                                    setEditForm({ ...editForm, password: (e.target as HTMLInputElement).value })
                                }
                                placeholder="Leave blank to keep current password"
                            />
                        </FormField>
                        <FormField id="edit-user-role" label="Role">
                            <FormSelect
                                id="edit-user-role"
                                value={editForm.role}
                                onChange={(e) =>
                                    setEditForm({ ...editForm, role: (e.target as HTMLSelectElement).value })
                                }
                            >
                                <option value="normal">Normal</option>
                                <option value="admin">Admin</option>
                            </FormSelect>
                        </FormField>
                        <div class="buttons">
                            <Button type="submit">Save</Button>
                            <SecondaryButton type="button" onClick={() => setEditingUserId(null)}>
                                Cancel
                            </SecondaryButton>
                        </div>
                    </form>
                </Dialog>
            )}

            {deleteUser && (
                <ConfirmDialog
                    title="Delete User"
                    message={`Delete ${deleteUser.firstName} ${deleteUser.lastName}?`}
                    confirmLabel="Delete"
                    confirmationText={deleteUser.email}
                    onConfirm={handleDelete}
                    onClose={() => setDeleteUser(null)}
                />
            )}

            {loadError && <div class="notification is-danger">{loadError}</div>}
            {loading && <div class="loading">Loading users…</div>}
            {!loading && !loadError && (
                <div class="table-wrap">
                    <table class="table">
                        <thead>
                            <tr>
                                <th>Name</th>
                                <th>Email</th>
                                <th>Role</th>
                                <th>Created</th>
                                <th></th>
                            </tr>
                        </thead>
                        <tbody>
                            {users.map((u) => (
                                <tr key={u.id}>
                                    <td>{`${u.firstName} ${u.lastName}`.trim()}</td>
                                    <td>{u.email}</td>
                                    <td class="role-badge">{capitalizeLabel(u.role)}</td>
                                    <td class="has-text-muted">{new Date(u.createdAt).toLocaleDateString()}</td>
                                    <td>
                                        <div class="buttons">
                                            <SecondaryButton class="is-small" onClick={() => startEditing(u)}>
                                                <PencilIcon class="is-sm" />
                                                Edit
                                            </SecondaryButton>
                                            <DangerButton class="is-small" onClick={() => setDeleteUser(u)}>
                                                <DeleteIcon class="is-sm" />
                                                Delete
                                            </DangerButton>
                                        </div>
                                    </td>
                                </tr>
                            ))}
                        </tbody>
                    </table>
                </div>
            )}
        </div>
    );
}
