/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import { useEffect, useState } from 'preact/hooks';
import type { Report } from '../src-gen/api.ts';
import { SettingsLayout } from '../components/settings-layout.tsx';
import { $authUser } from '../services/auth.ts';
import { changePassword, updateCurrentUser } from '../services/users.ts';

export function SettingsAccountPage() {
    const user = $authUser.value!;
    const [firstName, setFirstName] = useState(user.firstName);
    const [lastName, setLastName] = useState(user.lastName);
    const [email, setEmail] = useState(user.email);
    const [profileLoading, setProfileLoading] = useState(false);
    const [profileSaved, setProfileSaved] = useState(false);
    const [profileReport, setProfileReport] = useState<Report | null>(null);

    const [oldPassword, setOldPassword] = useState('');
    const [newPassword, setNewPassword] = useState('');
    const [confirmPassword, setConfirmPassword] = useState('');
    const [passwordLoading, setPasswordLoading] = useState(false);
    const [passwordChanged, setPasswordChanged] = useState(false);
    const [passwordReport, setPasswordReport] = useState<Report | null>(null);

    useEffect(() => {
        document.title = 'PlaatDeploy - Settings';
    }, []);

    async function handleProfileSubmit(event: Event) {
        event.preventDefault();
        setProfileLoading(true);
        setProfileSaved(false);
        setProfileReport(null);
        const { data, report } = await updateCurrentUser(user.id, { firstName, lastName, email });
        setProfileLoading(false);
        if (data) {
            $authUser.value = data;
            setProfileSaved(true);
        } else {
            setProfileReport(report ?? { profile: ['Failed to update profile'] });
        }
    }

    async function handlePasswordSubmit(event: Event) {
        event.preventDefault();
        setPasswordChanged(false);
        if (newPassword !== confirmPassword) {
            setPasswordReport({ confirm_password: ['Passwords do not match'] });
            return;
        }

        setPasswordLoading(true);
        setPasswordReport(null);
        const { ok, report } = await changePassword(user.id, oldPassword, newPassword);
        setPasswordLoading(false);
        if (ok) {
            setPasswordChanged(true);
            setOldPassword('');
            setNewPassword('');
            setConfirmPassword('');
        } else {
            setPasswordReport(report ?? { password: ['Failed to change password'] });
        }
    }

    return (
        <SettingsLayout>
            <div class="page">
                <div class="page-header">
                    <div>
                        <h1>Settings</h1>
                        <p class="page-subtitle">Update your profile details and password.</p>
                    </div>
                </div>

                <div class="stack" style="max-width: 720px;">
                    <div class="card">
                        <h2>Profile</h2>
                        <p class="card-meta" style="margin-bottom: 16px;">
                            Manage the name and email shown for your account.
                        </p>
                        {profileSaved && <div class="alert alert-success">Changes saved.</div>}
                        {profileReport && <div class="alert alert-error">Please fix the highlighted fields.</div>}
                        <form onSubmit={handleProfileSubmit}>
                            <div class="two-column-grid" style="grid-template-columns: repeat(2, minmax(0, 1fr));">
                                <div class="form-group">
                                    <label>First Name</label>
                                    <input
                                        value={firstName}
                                        onInput={(event) => setFirstName((event.target as HTMLInputElement).value)}
                                        required
                                    />
                                </div>
                                <div class="form-group">
                                    <label>Last Name</label>
                                    <input
                                        value={lastName}
                                        onInput={(event) => setLastName((event.target as HTMLInputElement).value)}
                                        required
                                    />
                                </div>
                            </div>
                            <div class="form-group">
                                <label>Email</label>
                                <input
                                    type="email"
                                    value={email}
                                    onInput={(event) => setEmail((event.target as HTMLInputElement).value)}
                                    required
                                />
                                {profileReport?.email?.[0] && (
                                    <p class="card-meta" style="color: #991b1b; margin-top: 6px;">
                                        {profileReport.email[0]}
                                    </p>
                                )}
                            </div>
                            <div class="dialog-actions">
                                <button class="btn btn-primary" type="submit" disabled={profileLoading}>
                                    {profileLoading ? 'Saving...' : 'Save changes'}
                                </button>
                            </div>
                        </form>
                    </div>

                    <div class="card">
                        <h2>Password</h2>
                        <p class="card-meta" style="margin-bottom: 16px;">
                            Change your password for future logins.
                        </p>
                        {passwordChanged && <div class="alert alert-success">Password changed.</div>}
                        {passwordReport && <div class="alert alert-error">Please fix the highlighted fields.</div>}
                        <form onSubmit={handlePasswordSubmit}>
                            <div class="form-group">
                                <label>Current Password</label>
                                <input
                                    type="password"
                                    value={oldPassword}
                                    onInput={(event) => setOldPassword((event.target as HTMLInputElement).value)}
                                    required
                                />
                                {passwordReport?.old_password?.[0] && (
                                    <p class="card-meta" style="color: #991b1b; margin-top: 6px;">
                                        {passwordReport.old_password[0]}
                                    </p>
                                )}
                            </div>
                            <div class="form-group">
                                <label>New Password</label>
                                <input
                                    type="password"
                                    value={newPassword}
                                    onInput={(event) => setNewPassword((event.target as HTMLInputElement).value)}
                                    required
                                />
                                {passwordReport?.new_password?.[0] && (
                                    <p class="card-meta" style="color: #991b1b; margin-top: 6px;">
                                        {passwordReport.new_password[0]}
                                    </p>
                                )}
                            </div>
                            <div class="form-group">
                                <label>Confirm Password</label>
                                <input
                                    type="password"
                                    value={confirmPassword}
                                    onInput={(event) => setConfirmPassword((event.target as HTMLInputElement).value)}
                                    required
                                />
                                {passwordReport?.confirm_password?.[0] && (
                                    <p class="card-meta" style="color: #991b1b; margin-top: 6px;">
                                        {passwordReport.confirm_password[0]}
                                    </p>
                                )}
                            </div>
                            <div class="dialog-actions">
                                <button class="btn btn-primary" type="submit" disabled={passwordLoading}>
                                    {passwordLoading ? 'Changing...' : 'Change password'}
                                </button>
                            </div>
                        </form>
                    </div>
                </div>
            </div>
        </SettingsLayout>
    );
}
