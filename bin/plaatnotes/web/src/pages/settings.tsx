/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import { useEffect, useState } from 'preact/hooks';
import { $token, $user, getAuthHeaders } from '../auth.ts';
import { Link } from '../router.tsx';
import { API_URL } from '../consts.ts';
import { type User, type Report } from '../../src-gen/api.ts';

export function Settings() {
    const [activeTab, setActiveTab] = useState('profile');
    const [user, setUser] = useState<User | null>(null);
    const [loading, setLoading] = useState(true);

    const [firstName, setFirstName] = useState('');
    const [lastName, setLastName] = useState('');
    const [email, setEmail] = useState('');
    const [updateErrors, setUpdateErrors] = useState<Record<string, string>>({});
    const [updateSuccess, setUpdateSuccess] = useState(false);
    const [updateLoading, setUpdateLoading] = useState(false);

    const [oldPassword, setOldPassword] = useState('');
    const [newPassword, setNewPassword] = useState('');
    const [confirmPassword, setConfirmPassword] = useState('');
    const [passwordErrors, setPasswordErrors] = useState<Record<string, string>>({});
    const [passwordSuccess, setPasswordSuccess] = useState(false);
    const [passwordLoading, setPasswordLoading] = useState(false);

    useEffect(() => {
        document.title = 'PlaatNotes - Settings';
        loadUser();
    }, []);

    async function loadUser() {
        try {
            if ($user.value) {
                setUser($user.value);
                setFirstName($user.value.firstName);
                setLastName($user.value.lastName);
                setEmail($user.value.email);
            }
        } finally {
            setLoading(false);
        }
    }

    async function handleUpdateUser(e: SubmitEvent) {
        e.preventDefault();
        setUpdateErrors({});
        setUpdateSuccess(false);
        setUpdateLoading(true);

        try {
            const res = await fetch(`${API_URL}/users/${user?.id}`, {
                method: 'PUT',
                headers: getAuthHeaders(),
                body: new URLSearchParams({ firstName, lastName, email }),
            });

            if (res.ok) {
                const updated: User = await res.json();
                setUser(updated);
                $user.value = updated;
                setUpdateSuccess(true);
                setTimeout(() => setUpdateSuccess(false), 3000);
            } else {
                const errors: Report = await res.json();
                setUpdateErrors(
                    Object.entries(errors).reduce((acc, [key, msgs]) => {
                        acc[key] = msgs.join(', ');
                        return acc;
                    }, {} as Record<string, string>)
                );
            }
        } finally {
            setUpdateLoading(false);
        }
    }

    async function handleChangePassword(e: SubmitEvent) {
        e.preventDefault();
        setPasswordErrors({});
        setPasswordSuccess(false);

        if (newPassword !== confirmPassword) {
            setPasswordErrors({ newPassword: 'Passwords do not match' });
            return;
        }

        if (newPassword.length < 8) {
            setPasswordErrors({ newPassword: 'Password must be at least 8 characters' });
            return;
        }

        setPasswordLoading(true);
        try {
            const res = await fetch(`${API_URL}/users/${user?.id}/change-password`, {
                method: 'POST',
                headers: getAuthHeaders(),
                body: new URLSearchParams({ oldPassword, newPassword }),
            });

            if (res.ok) {
                setOldPassword('');
                setNewPassword('');
                setConfirmPassword('');
                setPasswordSuccess(true);
                setTimeout(() => setPasswordSuccess(false), 3000);
            } else {
                const errors: Report = await res.json();
                setPasswordErrors(
                    Object.entries(errors).reduce((acc, [key, msgs]) => {
                        acc[key] = msgs.join(', ');
                        return acc;
                    }, {} as Record<string, string>)
                );
            }
        } finally {
            setPasswordLoading(false);
        }
    }

    if (loading) {
        return (
            <div class="container">
                <h1 class="title">Settings</h1>
                <p>Loading...</p>
            </div>
        );
    }

    return (
        <div class="container">
            <div class="buttons mb-4">
                <Link href="/" class="button">
                    <svg class="icon" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24">
                        <path d="M20,11V13H8L13.5,18.5L12.08,19.92L4.16,12L12.08,4.08L13.5,5.5L8,11H20Z" />
                    </svg>
                    Back
                </Link>
            </div>

            <h1 class="title">Settings</h1>

            <div class="tabs">
                <ul>
                    <li class={activeTab === 'profile' ? 'is-active' : ''}>
                        <a onClick={() => setActiveTab('profile')}>Profile</a>
                    </li>
                    <li class={activeTab === 'password' ? 'is-active' : ''}>
                        <a onClick={() => setActiveTab('password')}>Change Password</a>
                    </li>
                </ul>
            </div>

            {activeTab === 'profile' && (
                <div class="box">
                    <h2 class="subtitle">User Details</h2>

                    {updateSuccess && (
                        <div class="notification is-success is-light mb-4">
                            <button class="delete" onClick={() => setUpdateSuccess(false)} />
                            Profile updated successfully!
                        </div>
                    )}

                    {Object.keys(updateErrors).length > 0 && (
                        <div class="notification is-danger is-light mb-4">
                            <button class="delete" onClick={() => setUpdateErrors({})} />
                            {Object.entries(updateErrors).map(([key, msg]) => (
                                <div key={key}>
                                    <strong>{key}:</strong> {msg}
                                </div>
                            ))}
                        </div>
                    )}

                    <form onSubmit={handleUpdateUser}>
                        <div class="field">
                            <label class="label">First Name</label>
                            <div class="control">
                                <input
                                    class={`input ${updateErrors.firstName ? 'is-danger' : ''}`}
                                    type="text"
                                    value={firstName}
                                    onInput={(e) => setFirstName((e.target as HTMLInputElement).value)}
                                />
                            </div>
                            {updateErrors.firstName && (
                                <p class="help is-danger">{updateErrors.firstName}</p>
                            )}
                        </div>

                        <div class="field">
                            <label class="label">Last Name</label>
                            <div class="control">
                                <input
                                    class={`input ${updateErrors.lastName ? 'is-danger' : ''}`}
                                    type="text"
                                    value={lastName}
                                    onInput={(e) => setLastName((e.target as HTMLInputElement).value)}
                                />
                            </div>
                            {updateErrors.lastName && (
                                <p class="help is-danger">{updateErrors.lastName}</p>
                            )}
                        </div>

                        <div class="field">
                            <label class="label">Email</label>
                            <div class="control">
                                <input
                                    class={`input ${updateErrors.email ? 'is-danger' : ''}`}
                                    type="email"
                                    value={email}
                                    onInput={(e) => setEmail((e.target as HTMLInputElement).value)}
                                />
                            </div>
                            {updateErrors.email && <p class="help is-danger">{updateErrors.email}</p>}
                        </div>

                        <div class="field">
                            <div class="control">
                                <button
                                    class={`button is-link ${updateLoading ? 'is-loading' : ''}`}
                                    type="submit"
                                    disabled={updateLoading}
                                >
                                    Save Changes
                                </button>
                            </div>
                        </div>
                    </form>
                </div>
            )}

            {activeTab === 'password' && (
                <div class="box">
                    <h2 class="subtitle">Change Password</h2>

                    {passwordSuccess && (
                        <div class="notification is-success is-light mb-4">
                            <button class="delete" onClick={() => setPasswordSuccess(false)} />
                            Password changed successfully!
                        </div>
                    )}

                    {Object.keys(passwordErrors).length > 0 && (
                        <div class="notification is-danger is-light mb-4">
                            <button class="delete" onClick={() => setPasswordErrors({})} />
                            {Object.entries(passwordErrors).map(([key, msg]) => (
                                <div key={key}>
                                    <strong>{key}:</strong> {msg}
                                </div>
                            ))}
                        </div>
                    )}

                    <form onSubmit={handleChangePassword}>
                        <div class="field">
                            <label class="label">Current Password</label>
                            <div class="control">
                                <input
                                    class={`input ${passwordErrors.oldPassword ? 'is-danger' : ''}`}
                                    type="password"
                                    value={oldPassword}
                                    onInput={(e) => setOldPassword((e.target as HTMLInputElement).value)}
                                />
                            </div>
                            {passwordErrors.oldPassword && (
                                <p class="help is-danger">{passwordErrors.oldPassword}</p>
                            )}
                        </div>

                        <div class="field">
                            <label class="label">New Password</label>
                            <div class="control">
                                <input
                                    class={`input ${passwordErrors.newPassword ? 'is-danger' : ''}`}
                                    type="password"
                                    value={newPassword}
                                    onInput={(e) => setNewPassword((e.target as HTMLInputElement).value)}
                                />
                            </div>
                            {passwordErrors.newPassword && (
                                <p class="help is-danger">{passwordErrors.newPassword}</p>
                            )}
                        </div>

                        <div class="field">
                            <label class="label">Confirm Password</label>
                            <div class="control">
                                <input
                                    class={`input ${passwordErrors.confirmPassword ? 'is-danger' : ''}`}
                                    type="password"
                                    value={confirmPassword}
                                    onInput={(e) =>
                                        setConfirmPassword((e.target as HTMLInputElement).value)
                                    }
                                />
                            </div>
                            {passwordErrors.confirmPassword && (
                                <p class="help is-danger">{passwordErrors.confirmPassword}</p>
                            )}
                        </div>

                        <div class="field">
                            <div class="control">
                                <button
                                    class={`button is-link ${passwordLoading ? 'is-loading' : ''}`}
                                    type="submit"
                                    disabled={passwordLoading}
                                >
                                    Change Password
                                </button>
                            </div>
                        </div>
                    </form>
                </div>
            )}
        </div>
    );
}
