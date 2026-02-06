/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import { useEffect, useState } from 'preact/hooks';
import { useSignal } from '@preact/signals';
import { $authUser } from '../services/auth.service.ts';
import { UserService } from '../services/user.service.ts';
import { ThemeService } from '../services/theme.service.ts';
import { Navbar } from '../components/navbar.tsx';
import { UserTheme } from '../../src-gen/api.ts';

export function UserSettings() {
    const authUser = useSignal($authUser.value);
    const [firstName, setFirstName] = useState<string>('');
    const [lastName, setLastName] = useState<string>('');
    const [email, setEmail] = useState<string>('');
    const [theme, setTheme] = useState<UserTheme>(UserTheme.SYSTEM);
    const [oldPassword, setOldPassword] = useState<string>('');
    const [newPassword, setNewPassword] = useState<string>('');
    const [confirmPassword, setConfirmPassword] = useState<string>('');
    const [successMessage, setSuccessMessage] = useState<string>('');
    const [errorMessage, setErrorMessage] = useState<string>('');
    const [isLoading, setIsLoading] = useState<boolean>(false);
    const [activeTab, setActiveTab] = useState<'profile' | 'password'>('profile');

    useEffect(() => {
        document.title = 'PlaatNotes - Settings';
    }, []);

    useEffect(() => {
        const unsub = $authUser.subscribe((v) => {
            authUser.value = v;
            if (v) {
                setFirstName(v.firstName || '');
                setLastName(v.lastName || '');
                setEmail(v.email || '');
                setTheme(v.theme || 'system');
            }
        });
        return unsub;
    }, []);

    useEffect(() => {
        if (authUser.value) {
            setFirstName(authUser.value.firstName || '');
            setLastName(authUser.value.lastName || '');
            setEmail(authUser.value.email || '');
            setTheme(authUser.value.theme || 'system');
        }
    }, [authUser.value]);

    async function handleUpdateProfile(event: SubmitEvent) {
        event.preventDefault();
        setSuccessMessage('');
        setErrorMessage('');
        setIsLoading(true);

        const success = await UserService.getInstance().updateUser(
            authUser.value!.id,
            firstName,
            lastName,
            email,
            theme,
        );
        if (success) {
            ThemeService.applyTheme(theme);
            setSuccessMessage('Profile updated successfully');
        } else {
            setErrorMessage('Failed to update profile');
        }
        setIsLoading(false);
    }

    async function handleChangePassword(event: SubmitEvent) {
        event.preventDefault();
        setSuccessMessage('');
        setErrorMessage('');

        if (newPassword !== confirmPassword) {
            setErrorMessage('Passwords do not match');
            return;
        }

        if (newPassword.length < 6) {
            setErrorMessage('New password must be at least 6 characters');
            return;
        }

        setIsLoading(true);
        const success = await UserService.getInstance().changePassword(authUser.value!.id, oldPassword, newPassword);
        if (success) {
            setSuccessMessage('Password changed successfully');
            setOldPassword('');
            setNewPassword('');
            setConfirmPassword('');
        } else {
            setErrorMessage('Failed to change password. Please check your old password.');
        }
        setIsLoading(false);
    }

    if (!authUser.value) {
        return null;
    }

    return (
        <>
            <Navbar />
            <section class="section">
                <div class="container" style={{ maxWidth: '600px' }}>
                    <h1 class="title">Settings</h1>

                    <div class="tabs is-boxed">
                        <ul>
                            <li class={activeTab === 'profile' ? 'is-active' : ''}>
                                <a onClick={() => setActiveTab('profile')}>Profile</a>
                            </li>
                            <li class={activeTab === 'password' ? 'is-active' : ''}>
                                <a onClick={() => setActiveTab('password')}>Change Password</a>
                            </li>
                        </ul>
                    </div>

                    {successMessage && <div class="notification is-success">{successMessage}</div>}
                    {errorMessage && <div class="notification is-danger">{errorMessage}</div>}

                    {activeTab === 'profile' && (
                        <div class="box">
                            <h2 class="subtitle">Update Profile</h2>

                            <form onSubmit={handleUpdateProfile}>
                                <div class="field">
                                    <label class="label">First Name</label>
                                    <div class="control">
                                        <input
                                            class="input"
                                            type="text"
                                            placeholder="First Name"
                                            value={firstName}
                                            onInput={(e) => setFirstName((e.target as HTMLInputElement).value)}
                                        />
                                    </div>
                                </div>

                                <div class="field">
                                    <label class="label">Last Name</label>
                                    <div class="control">
                                        <input
                                            class="input"
                                            type="text"
                                            placeholder="Last Name"
                                            value={lastName}
                                            onInput={(e) => setLastName((e.target as HTMLInputElement).value)}
                                        />
                                    </div>
                                </div>

                                <div class="field">
                                    <label class="label">Email</label>
                                    <div class="control">
                                        <input
                                            class="input"
                                            type="email"
                                            placeholder="your@email.com"
                                            value={email}
                                            onInput={(e) => setEmail((e.target as HTMLInputElement).value)}
                                            required
                                        />
                                    </div>
                                </div>

                                <div class="field">
                                    <label class="label">Theme</label>
                                    <div class="control">
                                        <div class="select">
                                            <select
                                                value={theme}
                                                onChange={(e) =>
                                                    setTheme((e.target as HTMLSelectElement).value as UserTheme)
                                                }
                                            >
                                                <option value="system">System Default</option>
                                                <option value="light">Light</option>
                                                <option value="dark">Dark</option>
                                            </select>
                                        </div>
                                    </div>
                                </div>

                                <div class="field is-grouped">
                                    <div class="control">
                                        <button class="button is-link" type="submit" disabled={isLoading}>
                                            {isLoading ? 'Saving...' : 'Save Changes'}
                                        </button>
                                    </div>
                                </div>
                            </form>
                        </div>
                    )}

                    {activeTab === 'password' && (
                        <div class="box">
                            <h2 class="subtitle">Change Password</h2>

                            <form onSubmit={handleChangePassword}>
                                <div class="field">
                                    <label class="label">Current Password</label>
                                    <div class="control">
                                        <input
                                            class="input"
                                            type="password"
                                            placeholder="Current Password"
                                            value={oldPassword}
                                            onInput={(e) => setOldPassword((e.target as HTMLInputElement).value)}
                                            required
                                        />
                                    </div>
                                </div>

                                <div class="field">
                                    <label class="label">New Password</label>
                                    <div class="control">
                                        <input
                                            class="input"
                                            type="password"
                                            placeholder="New Password"
                                            value={newPassword}
                                            onInput={(e) => setNewPassword((e.target as HTMLInputElement).value)}
                                            required
                                        />
                                    </div>
                                </div>

                                <div class="field">
                                    <label class="label">Confirm Password</label>
                                    <div class="control">
                                        <input
                                            class="input"
                                            type="password"
                                            placeholder="Confirm Password"
                                            value={confirmPassword}
                                            onInput={(e) => setConfirmPassword((e.target as HTMLInputElement).value)}
                                            required
                                        />
                                    </div>
                                </div>

                                <div class="field is-grouped">
                                    <div class="control">
                                        <button class="button is-link" type="submit" disabled={isLoading}>
                                            {isLoading ? 'Changing...' : 'Change Password'}
                                        </button>
                                    </div>
                                </div>
                            </form>
                        </div>
                    )}
                </div>
            </section>
        </>
    );
}
