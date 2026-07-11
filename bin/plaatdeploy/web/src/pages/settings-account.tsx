/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import { useState } from 'preact/hooks';
import type { Report } from '../src-gen/api.ts';
import { SettingsLayout } from '../components/settings-layout.tsx';
import { Button } from '../components/button.tsx';
import { Card } from '../components/card.tsx';
import { FormField } from '../components/form.tsx';
import { FormInput } from '../components/input.tsx';
import { $authUser } from '../services/auth.ts';
import { changePassword, updateCurrentUser } from '../services/users.ts';
import { useDocumentTitle } from '../utils.ts';

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
    useDocumentTitle('Settings');

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

                <div class="stack is-narrow">
                    <Card>
                        <div class="card-header">
                            <div>
                                <h2>Profile</h2>
                                <p class="card-meta">Manage the name and email shown for your account.</p>
                            </div>
                        </div>
                        {profileSaved && <div class="notification is-success">Changes saved.</div>}
                        {profileReport && <div class="notification is-danger">Please fix the highlighted fields.</div>}
                        <form onSubmit={handleProfileSubmit}>
                            <div class="field-row">
                                <FormField id="account-first-name" label="First Name">
                                    <FormInput
                                        id="account-first-name"
                                        value={firstName}
                                        onInput={(event) => setFirstName((event.target as HTMLInputElement).value)}
                                        required
                                    />
                                </FormField>
                                <FormField id="account-last-name" label="Last Name">
                                    <FormInput
                                        id="account-last-name"
                                        value={lastName}
                                        onInput={(event) => setLastName((event.target as HTMLInputElement).value)}
                                        required
                                    />
                                </FormField>
                            </div>
                            <FormField id="account-email" label="Email" error={profileReport?.email?.[0]}>
                                <FormInput
                                    id="account-email"
                                    type="email"
                                    value={email}
                                    onInput={(event) => setEmail((event.target as HTMLInputElement).value)}
                                    required
                                />
                            </FormField>
                            <div class="buttons">
                                <Button type="submit" disabled={profileLoading}>
                                    {profileLoading ? 'Saving…' : 'Save changes'}
                                </Button>
                            </div>
                        </form>
                    </Card>

                    <Card>
                        <div class="card-header">
                            <div>
                                <h2>Password</h2>
                                <p class="card-meta">Change your password for future logins.</p>
                            </div>
                        </div>
                        {passwordChanged && <div class="notification is-success">Password changed.</div>}
                        {passwordReport && <div class="notification is-danger">Please fix the highlighted fields.</div>}
                        <form onSubmit={handlePasswordSubmit}>
                            <FormField
                                id="account-old-password"
                                label="Current Password"
                                error={passwordReport?.old_password?.[0]}
                            >
                                <FormInput
                                    id="account-old-password"
                                    type="password"
                                    value={oldPassword}
                                    onInput={(event) => setOldPassword((event.target as HTMLInputElement).value)}
                                    required
                                />
                            </FormField>
                            <FormField
                                id="account-new-password"
                                label="New Password"
                                error={passwordReport?.new_password?.[0]}
                            >
                                <FormInput
                                    id="account-new-password"
                                    type="password"
                                    value={newPassword}
                                    onInput={(event) => setNewPassword((event.target as HTMLInputElement).value)}
                                    required
                                />
                            </FormField>
                            <FormField
                                id="account-confirm-password"
                                label="Confirm Password"
                                error={passwordReport?.confirm_password?.[0]}
                            >
                                <FormInput
                                    id="account-confirm-password"
                                    type="password"
                                    value={confirmPassword}
                                    onInput={(event) => setConfirmPassword((event.target as HTMLInputElement).value)}
                                    required
                                />
                            </FormField>
                            <div class="buttons">
                                <Button type="submit" disabled={passwordLoading}>
                                    {passwordLoading ? 'Changing…' : 'Change password'}
                                </Button>
                            </div>
                        </form>
                    </Card>
                </div>
            </div>
        </SettingsLayout>
    );
}
