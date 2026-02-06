/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import { useEffect, useState } from 'preact/hooks';
import { Link } from '../../router.tsx';
import { $authToken, $authUser } from '../../services/auth.service.ts';
import { UserService } from '../../services/user.service.ts';
import { useSignal } from '@preact/signals';
import { UserRole, type User } from '../../../src-gen/api.ts';
import { Navbar } from '../../components/navbar.tsx';

export function AdminUsers() {
    const authToken = useSignal($authToken.value);
    const authUser = useSignal($authUser.value);
    const [users, setUsers] = useState<User[]>([]);
    const [showCreateForm, setShowCreateForm] = useState<boolean>(false);
    const [formData, setFormData] = useState({
        firstName: '',
        lastName: '',
        email: '',
        password: '',
        role: UserRole.NORMAL,
    });
    const [error, setError] = useState<string>('');

    useEffect(() => {
        document.title = 'PlaatNotes - Admin Users';
        const unsubToken = $authToken.subscribe((v) => (authToken.value = v));
        const unsubUser = $authUser.subscribe((v) => (authUser.value = v));
        return () => {
            unsubToken();
            unsubUser();
        };
    }, []);

    // @ts-ignore
    useEffect(async () => {
        if (!authToken.value) return;

        const loadedUsers = await UserService.getInstance().getAllUsers();
        setUsers(loadedUsers);
    }, [authToken.value]);

    async function createUser() {
        setError('');
        const newUser = await UserService.getInstance().createUser(
            formData.firstName,
            formData.lastName,
            formData.email,
            formData.password,
            formData.role,
        );

        if (newUser) {
            setUsers([...users, newUser]);
            setFormData({ firstName: '', lastName: '', email: '', password: '', role: UserRole.NORMAL });
            setShowCreateForm(false);
        } else {
            setError('Failed to create user');
        }
    }

    async function deleteUser(id: string) {
        if (!confirm('Are you sure you want to delete this user?')) return;

        const success = await UserService.getInstance().deleteUser(id);
        if (success) {
            setUsers(users.filter((u) => u.id !== id));
        } else {
            setError('Failed to delete user');
        }
    }

    return (
        <>
            <Navbar />
            <section class="section">
                <div class="container">
                    <h1 class="title">Manage Users</h1>
                    <div class="buttons">
                        <Link href="/" class="button">
                            <span class="icon">
                                <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24">
                                    <path d="M20,11V13H8L13.5,18.5L12.08,19.92L4.16,12L12.08,4.08L13.5,5.5L8,11H20Z" />
                                </svg>
                            </span>
                            <span>Back</span>
                        </Link>
                    </div>

                    {error && <div class="notification is-danger">{error}</div>}

                    <button class="button is-link" onClick={() => setShowCreateForm(!showCreateForm)}>
                        {showCreateForm ? 'Cancel' : 'Create User'}
                    </button>

                    {showCreateForm && (
                        <div class="box" style={{ marginTop: '1rem' }}>
                            <h3 class="subtitle">Create New User</h3>
                            <div class="field">
                                <label class="label">First Name</label>
                                <input
                                    class="input"
                                    value={formData.firstName}
                                    onInput={(e) =>
                                        setFormData({ ...formData, firstName: (e.target as HTMLInputElement).value })
                                    }
                                />
                            </div>
                            <div class="field">
                                <label class="label">Last Name</label>
                                <input
                                    class="input"
                                    value={formData.lastName}
                                    onInput={(e) =>
                                        setFormData({ ...formData, lastName: (e.target as HTMLInputElement).value })
                                    }
                                />
                            </div>
                            <div class="field">
                                <label class="label">Email</label>
                                <input
                                    class="input"
                                    type="email"
                                    value={formData.email}
                                    onInput={(e) =>
                                        setFormData({ ...formData, email: (e.target as HTMLInputElement).value })
                                    }
                                />
                            </div>
                            <div class="field">
                                <label class="label">Password</label>
                                <input
                                    class="input"
                                    type="password"
                                    value={formData.password}
                                    onInput={(e) =>
                                        setFormData({ ...formData, password: (e.target as HTMLInputElement).value })
                                    }
                                />
                            </div>
                            <div class="field">
                                <label class="label">Role</label>
                                <div class="select">
                                    <select
                                        value={formData.role}
                                        onChange={(e) =>
                                            setFormData({
                                                ...formData,
                                                role: (e.target as HTMLSelectElement).value as UserRole,
                                            })
                                        }
                                    >
                                        <option value="normal">Normal</option>
                                        <option value="admin">Admin</option>
                                    </select>
                                </div>
                            </div>
                            <button class="button is-success" onClick={createUser}>
                                Create
                            </button>
                        </div>
                    )}

                    <table class="table is-fullwidth" style={{ marginTop: '1rem' }}>
                        <thead>
                            <tr>
                                <th>Name</th>
                                <th>Email</th>
                                <th>Role</th>
                                <th>Created</th>
                                <th>Actions</th>
                            </tr>
                        </thead>
                        <tbody>
                            {users.map((user) => (
                                <tr key={user.id}>
                                    <td>
                                        {user.firstName} {user.lastName}
                                    </td>
                                    <td>{user.email}</td>
                                    <td>
                                        <span class={`tag ${user.role === UserRole.ADMIN ? 'is-danger' : 'is-info'}`}>
                                            {user.role}
                                        </span>
                                    </td>
                                    <td>{new Date(user.createdAt).toLocaleDateString()}</td>
                                    <td>
                                        {user.id !== authUser.value?.id && (
                                            <button
                                                class="button is-small is-danger"
                                                onClick={() => deleteUser(user.id)}
                                            >
                                                Delete
                                            </button>
                                        )}
                                    </td>
                                </tr>
                            ))}
                        </tbody>
                    </table>
                </div>
            </section>
        </>
    );
}
