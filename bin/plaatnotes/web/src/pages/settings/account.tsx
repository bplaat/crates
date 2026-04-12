/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import { useEffect, useState } from 'preact/hooks';
import { type Report, type UserTheme } from '../../../src-gen/api.ts';
import { Button } from '../../components/button.tsx';
import { Card } from '../../components/card.tsx';
import { FormActions, FormField, FormMessage } from '../../components/form.tsx';
import { FormInput, FormSelect } from '../../components/input.tsx';
import { SettingsLayout } from '../../components/settings-layout.tsx';
import { $authUser } from '../../services/auth.service.ts';
import { t } from '../../services/i18n.service.ts';
import { changePassword, updateUser } from '../../services/users.service.ts';

export function SettingsAccount() {
    const user = $authUser.value!;

    const [firstName, setFirstName] = useState(user.firstName);
    const [lastName, setLastName] = useState(user.lastName);
    const [email, setEmail] = useState(user.email);
    const [theme, setTheme] = useState<UserTheme>(user.theme);
    const [language, setLanguage] = useState(user.language);
    const [profileLoading, setProfileLoading] = useState(false);
    const [profileSaved, setProfileSaved] = useState(false);
    const [profileError, setProfileError] = useState(false);
    const [profileReport, setProfileReport] = useState<Report | null>(null);

    const [oldPassword, setOldPassword] = useState('');
    const [newPassword, setNewPassword] = useState('');
    const [confirmPassword, setConfirmPassword] = useState('');
    const [passwordLoading, setPasswordLoading] = useState(false);
    const [passwordChanged, setPasswordChanged] = useState(false);
    const [passwordReport, setPasswordReport] = useState<Report | null>(null);

    useEffect(() => {
        document.title = `PlaatNotes - ${t('page.settings')}`;
    }, []);

    async function handleProfileSubmit(e: SubmitEvent) {
        e.preventDefault();
        setProfileSaved(false);
        setProfileError(false);
        setProfileReport(null);
        setProfileLoading(true);
        const { data: updated, report } = await updateUser(user.id, {
            firstName,
            lastName,
            email,
            theme,
            language,
            role: user.role,
        });
        setProfileLoading(false);
        if (updated) {
            $authUser.value = updated;
            setProfileSaved(true);
        } else {
            setProfileError(true);
            setProfileReport(report);
        }
    }

    async function handlePasswordSubmit(e: SubmitEvent) {
        e.preventDefault();
        setPasswordChanged(false);
        setPasswordReport(null);
        setPasswordLoading(true);
        const { ok, report } = await changePassword(user.id, oldPassword, newPassword);
        setPasswordLoading(false);
        if (ok) {
            setPasswordChanged(true);
            setOldPassword('');
            setNewPassword('');
            setConfirmPassword('');
        } else {
            setPasswordReport(report);
        }
    }

    return (
        <SettingsLayout>
            <div class="max-w-2xl mx-auto px-4 py-8">
                <h1 class="text-xl font-medium text-gray-700 dark:text-gray-200 mb-6">{t('settings.account')}</h1>

                <div class="flex flex-col gap-6">
                    {/* Profile form */}
                    <Card>
                        <h2 class="text-base font-semibold text-gray-700 dark:text-gray-200 mb-5">
                            {t('settings.profile_heading')}
                        </h2>
                        <form onSubmit={handleProfileSubmit} class="flex flex-col gap-4">
                            <div class="grid grid-cols-2 gap-4">
                                <FormField
                                    id="firstName"
                                    label={t('settings.first_name')}
                                    error={profileReport?.['first_name']?.[0]}
                                >
                                    <FormInput
                                        id="firstName"
                                        type="text"
                                        required
                                        value={firstName}
                                        onInput={(e) => setFirstName((e.target as HTMLInputElement).value)}
                                    />
                                </FormField>
                                <FormField
                                    id="lastName"
                                    label={t('settings.last_name')}
                                    error={profileReport?.['last_name']?.[0]}
                                >
                                    <FormInput
                                        id="lastName"
                                        type="text"
                                        required
                                        value={lastName}
                                        onInput={(e) => setLastName((e.target as HTMLInputElement).value)}
                                    />
                                </FormField>
                            </div>

                            <FormField id="email" label={t('settings.email')} error={profileReport?.['email']?.[0]}>
                                <FormInput
                                    id="email"
                                    type="email"
                                    required
                                    value={email}
                                    onInput={(e) => setEmail((e.target as HTMLInputElement).value)}
                                />
                            </FormField>

                            <div class="grid grid-cols-2 gap-4">
                                <FormField id="theme" label={t('settings.theme')}>
                                    <FormSelect
                                        id="theme"
                                        value={theme}
                                        onChange={(e) => setTheme((e.target as HTMLSelectElement).value as UserTheme)}
                                    >
                                        <option value="system">{t('settings.theme_system')}</option>
                                        <option value="light">{t('settings.theme_light')}</option>
                                        <option value="dark">{t('settings.theme_dark')}</option>
                                    </FormSelect>
                                </FormField>
                                <FormField id="language" label={t('settings.language')}>
                                    <FormSelect
                                        id="language"
                                        value={language}
                                        onChange={(e) => setLanguage((e.target as HTMLSelectElement).value)}
                                    >
                                        <option value="en">{t('settings.language_en')}</option>
                                        <option value="nl">{t('settings.language_nl')}</option>
                                    </FormSelect>
                                </FormField>
                            </div>

                            <div class="flex flex-col gap-3 pt-1">
                                <div>
                                    <FormMessage type="success" message={profileSaved && t('settings.saved')} />
                                    <FormMessage type="error" message={profileError && t('form.errors_occurred')} />
                                </div>
                                <FormActions class="pt-0">
                                    <Button type="submit" disabled={profileLoading}>
                                        <span class="flex items-center gap-1.5">
                                            <svg class="w-4 h-4" viewBox="0 0 24 24" fill="currentColor">
                                                <path d="M17 3H5c-1.11 0-2 .9-2 2v14c0 1.1.89 2 2 2h14c1.1 0 2-.9 2-2V7l-4-4zm-5 16c-1.66 0-3-1.34-3-3s1.34-3 3-3 3 1.34 3 3-1.34 3-3 3zm3-10H5V5h10v4z" />
                                            </svg>
                                            {t('settings.save')}
                                        </span>
                                    </Button>
                                </FormActions>
                            </div>
                        </form>
                    </Card>

                    {/* Change password form */}
                    <Card>
                        <h2 class="text-base font-semibold text-gray-700 dark:text-gray-200 mb-5">
                            {t('settings.password_heading')}
                        </h2>
                        <form onSubmit={handlePasswordSubmit} class="flex flex-col gap-4">
                            <FormField
                                id="oldPassword"
                                label={t('settings.current_password')}
                                error={passwordReport?.['old_password']?.[0]}
                            >
                                <FormInput
                                    id="oldPassword"
                                    type="password"
                                    required
                                    placeholder="••••••••"
                                    value={oldPassword}
                                    onInput={(e) => setOldPassword((e.target as HTMLInputElement).value)}
                                />
                            </FormField>
                            <FormField
                                id="newPassword"
                                label={t('settings.new_password')}
                                error={passwordReport?.['new_password']?.[0]}
                            >
                                <FormInput
                                    id="newPassword"
                                    type="password"
                                    required
                                    placeholder="••••••••"
                                    value={newPassword}
                                    onInput={(e) => setNewPassword((e.target as HTMLInputElement).value)}
                                />
                            </FormField>
                            <FormField id="confirmPassword" label={t('settings.confirm_password')}>
                                <FormInput
                                    id="confirmPassword"
                                    type="password"
                                    required
                                    placeholder="••••••••"
                                    value={confirmPassword}
                                    onInput={(e) => setConfirmPassword((e.target as HTMLInputElement).value)}
                                />
                            </FormField>

                            <div class="flex flex-col gap-3 pt-1">
                                <div>
                                    <FormMessage
                                        type="success"
                                        message={passwordChanged && t('settings.password_changed')}
                                    />
                                    <FormMessage type="error" message={passwordReport && t('form.errors_occurred')} />
                                </div>
                                <FormActions class="pt-0">
                                    <Button type="submit" disabled={passwordLoading}>
                                        <span class="flex items-center gap-1.5">
                                            <svg class="w-4 h-4" viewBox="0 0 24 24" fill="currentColor">
                                                <path d="M18 8h-1V6c0-2.76-2.24-5-5-5S7 3.24 7 6v2H6c-1.1 0-2 .9-2 2v10c0 1.1.9 2 2 2h12c1.1 0 2-.9 2-2V10c0-1.1-.9-2-2-2zm-6 9c-1.1 0-2-.9-2-2s.9-2 2-2 2 .9 2 2-.9 2-2 2zm3.1-9H8.9V6c0-1.71 1.39-3.1 3.1-3.1 1.71 0 3.1 1.39 3.1 3.1v2z" />
                                            </svg>
                                            {t('settings.change_password')}
                                        </span>
                                    </Button>
                                </FormActions>
                            </div>
                        </form>
                    </Card>
                </div>
            </div>
        </SettingsLayout>
    );
}
