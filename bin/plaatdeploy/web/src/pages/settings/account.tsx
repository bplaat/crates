/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import {
    Button,
    Card,
    CardDescription,
    CardTitle,
    ContentSaveIcon,
    DangerButton,
    Form,
    FormActions,
    FormField,
    FormInput,
    FormMessage,
    FormRow,
    FormSelect,
    LockIcon,
    Page,
    PageTitle,
} from 'plaatui';
import { useEffect, useState } from 'preact/hooks';
import { type UserTheme } from '../../../src-gen/api.ts';
import { AppLayout } from '../../components/app-layout.tsx';
import { $authUser, authFetch } from '../../services/auth.service.ts';

export function Settings() {
    const user = $authUser.value!;
    const [firstName, setFirstName] = useState(user.firstName);
    const [lastName, setLastName] = useState(user.lastName);
    const [email, setEmail] = useState(user.email);
    const [theme, setTheme] = useState<UserTheme>(user.theme);
    const [profileMessage, setProfileMessage] = useState('');
    const [profileError, setProfileError] = useState(false);
    const [token, setToken] = useState('');
    const [githubMessage, setGithubMessage] = useState('');
    const [githubError, setGithubError] = useState(false);
    const [oldPassword, setOldPassword] = useState('');
    const [newPassword, setNewPassword] = useState('');
    const [passwordMessage, setPasswordMessage] = useState('');
    const [passwordError, setPasswordError] = useState(false);
    useEffect(() => {
        document.title = 'Plaat Deploy - Settings';
    }, []);
    async function saveProfile(event: SubmitEvent) {
        event.preventDefault();
        setProfileMessage('');
        setProfileError(false);
        try {
            const response = await authFetch('/api/account', {
                method: 'PUT',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ firstName, lastName, email, theme, language: user.language }),
            });
            if (response.ok) {
                $authUser.value = await response.json();
                setProfileMessage('Account saved.');
            } else {
                setProfileError(true);
                setProfileMessage(
                    response.status === 409 ? 'That email address is already in use.' : 'Could not save account.',
                );
            }
        } catch {
            setProfileError(true);
            setProfileMessage('Could not reach the server.');
        }
    }
    async function connectGithub(event: SubmitEvent) {
        event.preventDefault();
        setGithubMessage('');
        setGithubError(false);
        try {
            const response = await authFetch('/api/account/github', {
                method: 'PUT',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ token }),
            });
            if (!response.ok) throw new Error();
            const { login } = await response.json();
            $authUser.value = { ...user, githubLogin: login };
            setToken('');
            setGithubMessage(`Connected as @${login}.`);
        } catch {
            setGithubError(true);
            setGithubMessage('The token could not be validated.');
        }
    }
    async function disconnectGithub() {
        setGithubMessage('');
        setGithubError(false);
        try {
            if (!(await authFetch('/api/account/github', { method: 'DELETE' })).ok) throw new Error();
            $authUser.value = { ...user, githubLogin: undefined };
            setGithubMessage('GitHub disconnected.');
        } catch {
            setGithubError(true);
            setGithubMessage('Could not disconnect GitHub.');
        }
    }
    async function changePassword(event: SubmitEvent) {
        event.preventDefault();
        setPasswordMessage('');
        setPasswordError(false);
        try {
            const response = await authFetch('/api/account/password', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ oldPassword, newPassword }),
            });
            setPasswordError(!response.ok);
            setPasswordMessage(response.ok ? 'Password changed.' : 'Could not change password.');
            if (response.ok) {
                setOldPassword('');
                setNewPassword('');
            }
        } catch {
            setPasswordError(true);
            setPasswordMessage('Could not reach the server.');
        }
    }
    return (
        <AppLayout>
            <Page size="narrow">
                <PageTitle>Settings</PageTitle>
                <div class="settings-list">
                    <Card>
                        <CardTitle>Account</CardTitle>
                        <Form onSubmit={saveProfile}>
                            <FormRow>
                                <FormField id="firstName" label="First name">
                                    <FormInput
                                        id="firstName"
                                        required
                                        value={firstName}
                                        onInput={(e) => setFirstName(e.currentTarget.value)}
                                    />
                                </FormField>
                                <FormField id="lastName" label="Last name">
                                    <FormInput
                                        id="lastName"
                                        required
                                        value={lastName}
                                        onInput={(e) => setLastName(e.currentTarget.value)}
                                    />
                                </FormField>
                            </FormRow>
                            <FormField id="email" label="Email">
                                <FormInput
                                    id="email"
                                    type="email"
                                    required
                                    value={email}
                                    onInput={(e) => setEmail(e.currentTarget.value)}
                                />
                            </FormField>
                            <FormField id="theme" label="Theme">
                                <FormSelect
                                    id="theme"
                                    value={theme}
                                    onChange={(e) => setTheme(e.currentTarget.value as UserTheme)}
                                >
                                    <option value="system">System</option>
                                    <option value="light">Light</option>
                                    <option value="dark">Dark</option>
                                </FormSelect>
                            </FormField>
                            <FormMessage type={profileError ? 'error' : 'success'} message={profileMessage} />
                            <FormActions flush>
                                <Button type="submit">
                                    <ContentSaveIcon class="is-sm" />
                                    Save account
                                </Button>
                            </FormActions>
                        </Form>
                    </Card>
                    <Card>
                        <CardTitle>GitHub</CardTitle>
                        <CardDescription>
                            Use a fine-grained personal access token with Contents read, Webhooks read/write, and
                            Deployments read/write access.
                        </CardDescription>
                        {user.githubLogin ? (
                            <div class="connected-row">
                                <div>
                                    <strong>Connected as @{user.githubLogin}</strong>
                                    <p class="muted">The token is stored only on this server.</p>
                                </div>
                                <DangerButton onClick={() => void disconnectGithub()}>Disconnect</DangerButton>
                            </div>
                        ) : (
                            <Form onSubmit={connectGithub}>
                                <FormField id="githubToken" label="Personal access token">
                                    <FormInput
                                        id="githubToken"
                                        type="password"
                                        required
                                        value={token}
                                        placeholder="github_pat_..."
                                        onInput={(e) => setToken(e.currentTarget.value)}
                                    />
                                </FormField>
                                <FormActions flush>
                                    <Button type="submit">Connect GitHub</Button>
                                </FormActions>
                            </Form>
                        )}
                        <FormMessage type={githubError ? 'error' : 'success'} message={githubMessage} />
                    </Card>
                    <Card>
                        <CardTitle>Password</CardTitle>
                        <Form onSubmit={changePassword}>
                            <FormField id="oldPassword" label="Current password">
                                <FormInput
                                    id="oldPassword"
                                    type="password"
                                    required
                                    value={oldPassword}
                                    onInput={(e) => setOldPassword(e.currentTarget.value)}
                                />
                            </FormField>
                            <FormField id="newPassword" label="New password">
                                <FormInput
                                    id="newPassword"
                                    type="password"
                                    minLength={8}
                                    required
                                    value={newPassword}
                                    onInput={(e) => setNewPassword(e.currentTarget.value)}
                                />
                            </FormField>
                            <FormMessage type={passwordError ? 'error' : 'success'} message={passwordMessage} />
                            <FormActions flush>
                                <Button type="submit">
                                    <LockIcon class="is-sm" />
                                    Change password
                                </Button>
                            </FormActions>
                        </Form>
                    </Card>
                </div>
            </Page>
        </AppLayout>
    );
}
