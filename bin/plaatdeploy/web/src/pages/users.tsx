/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import { useEffect, useState } from 'preact/hooks';
import { useLocation } from 'wouter-preact';
import { ConfirmDialog, Dialog } from '../components/dialog.tsx';
import type { User, UserIndexResponse } from '../src-gen/api.ts';
import { $authUser, API_URL, authFetch, logout } from '../services/auth.ts';
import { capitalizeLabel } from '../utils.ts';

export function UsersPage() {
    const authUser = $authUser.value;
    const [, navigate] = useLocation();
    const [users, setUsers] = useState<User[]>([]);
    const [loading, setLoading] = useState(true);
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

    if (authUser?.role !== 'admin') {
        return <div class="empty">Access denied.</div>;
    }

    useEffect(() => {
        authFetch(`${API_URL}/users`)
            .then((r) => r.json())
            .then((data: UserIndexResponse) => {
                setUsers(data.data);
                setLoading(false);
            });
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

    return (
        <div class="page">
            <div class="page-header">
                <h1>Users</h1>
                <button class="btn btn-primary btn-sm" onClick={() => setCreating(true)}>
                    + New User
                </button>
            </div>

            <div class="card">
                <p class="card-meta">
                    Creating a user also creates a default "{`First Last's Team`}" team for them automatically.
                </p>
            </div>

            {creating && (
                <Dialog title="New User" onClose={() => setCreating(false)}>
                    {createError && <div class="alert alert-error">{createError}</div>}
                    <form onSubmit={handleCreate}>
                        <div class="form-group">
                            <label>First Name</label>
                            <input
                                value={form.first_name}
                                onInput={(e) => setForm({ ...form, first_name: (e.target as HTMLInputElement).value })}
                                required
                            />
                        </div>
                        <div class="form-group">
                            <label>Last Name</label>
                            <input
                                value={form.last_name}
                                onInput={(e) => setForm({ ...form, last_name: (e.target as HTMLInputElement).value })}
                                required
                            />
                        </div>
                        <div class="form-group">
                            <label>Email</label>
                            <input
                                type="email"
                                value={form.email}
                                onInput={(e) => setForm({ ...form, email: (e.target as HTMLInputElement).value })}
                                required
                            />
                        </div>
                        <div class="form-group">
                            <label>Password</label>
                            <input
                                type="password"
                                value={form.password}
                                onInput={(e) => setForm({ ...form, password: (e.target as HTMLInputElement).value })}
                                required
                            />
                        </div>
                        <div class="form-group">
                            <label>Role</label>
                            <select
                                value={form.role}
                                onChange={(e) => setForm({ ...form, role: (e.target as HTMLSelectElement).value })}
                            >
                                <option value="normal">Normal</option>
                                <option value="admin">Admin</option>
                            </select>
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

            {editingUserId && (
                <Dialog title="Edit User" onClose={() => setEditingUserId(null)}>
                    {editError && <div class="alert alert-error">{editError}</div>}
                    <form onSubmit={handleUpdate}>
                        <div class="form-group">
                            <label>First Name</label>
                            <input
                                value={editForm.first_name}
                                onInput={(e) =>
                                    setEditForm({ ...editForm, first_name: (e.target as HTMLInputElement).value })
                                }
                                required
                            />
                        </div>
                        <div class="form-group">
                            <label>Last Name</label>
                            <input
                                value={editForm.last_name}
                                onInput={(e) =>
                                    setEditForm({ ...editForm, last_name: (e.target as HTMLInputElement).value })
                                }
                                required
                            />
                        </div>
                        <div class="form-group">
                            <label>Email</label>
                            <input
                                type="email"
                                value={editForm.email}
                                onInput={(e) =>
                                    setEditForm({ ...editForm, email: (e.target as HTMLInputElement).value })
                                }
                                required
                            />
                        </div>
                        <div class="form-group">
                            <label>Password</label>
                            <input
                                type="password"
                                value={editForm.password}
                                onInput={(e) =>
                                    setEditForm({ ...editForm, password: (e.target as HTMLInputElement).value })
                                }
                                placeholder="Leave blank to keep current password"
                            />
                        </div>
                        <div class="form-group">
                            <label>Role</label>
                            <select
                                value={editForm.role}
                                onChange={(e) =>
                                    setEditForm({ ...editForm, role: (e.target as HTMLSelectElement).value })
                                }
                            >
                                <option value="normal">Normal</option>
                                <option value="admin">Admin</option>
                            </select>
                        </div>
                        <div class="dialog-actions">
                            <button class="btn btn-primary" type="submit">
                                Save
                            </button>
                            <button class="btn btn-secondary" type="button" onClick={() => setEditingUserId(null)}>
                                Cancel
                            </button>
                        </div>
                    </form>
                </Dialog>
            )}

            {deleteUser && (
                <ConfirmDialog
                    title="Delete User"
                    message={`Delete ${deleteUser.firstName} ${deleteUser.lastName}?`}
                    confirmLabel="Delete"
                    onConfirm={handleDelete}
                    onClose={() => setDeleteUser(null)}
                />
            )}

            {loading && <div class="loading">Loading users...</div>}
            {!loading && (
                <table>
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
                                <td class="muted-text">{new Date(u.createdAt).toLocaleDateString()}</td>
                                <td>
                                    <div class="section-actions">
                                        <button class="btn btn-secondary btn-sm" onClick={() => startEditing(u)}>
                                            Edit
                                        </button>
                                        <button class="btn btn-danger btn-sm" onClick={() => setDeleteUser(u)}>
                                            Delete
                                        </button>
                                    </div>
                                </td>
                            </tr>
                        ))}
                    </tbody>
                </table>
            )}
        </div>
    );
}
