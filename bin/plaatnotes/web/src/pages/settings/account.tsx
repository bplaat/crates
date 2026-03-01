/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import { useEffect, useState } from 'preact/hooks';
import { type UserTheme } from '../../../src-gen/api.ts';
import { Button, FormField, FormInput, FormSelect } from '../../components/form.tsx';
import { SettingsLayout } from '../../components/settings-layout.tsx';
import { $authUser } from '../../services/auth.service.ts';
import { t } from '../../services/i18n.service.ts';
import { changePassword, updateUser } from '../../services/users.service.ts';

const CARD_CLASS = 'bg-white dark:bg-zinc-800 rounded-2xl border border-gray-200 dark:border-zinc-700 shadow-sm p-6';

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

    const [oldPassword, setOldPassword] = useState('');
    const [newPassword, setNewPassword] = useState('');
    const [confirmPassword, setConfirmPassword] = useState('');
    const [passwordLoading, setPasswordLoading] = useState(false);
    const [passwordChanged, setPasswordChanged] = useState(false);
    const [passwordError, setPasswordError] = useState<string | null>(null);

    useEffect(() => {
        document.title = `PlaatNotes - ${t('page.settings')}`;
    }, []);

    async function handleProfileSubmit(e: SubmitEvent) {
        e.preventDefault();
        setProfileSaved(false);
        setProfileError(false);
        setProfileLoading(true);
        const updated = await updateUser(user.id, { firstName, lastName, email, theme, language, role: user.role });
        setProfileLoading(false);
        if (updated) {
            $authUser.value = updated;
            setProfileSaved(true);
        } else {
            setProfileError(true);
        }
    }

    async function handlePasswordSubmit(e: SubmitEvent) {
        e.preventDefault();
        setPasswordChanged(false);
        setPasswordError(null);
        if (newPassword !== confirmPassword) {
            setPasswordError(t('settings.password_mismatch'));
            return;
        }
        setPasswordLoading(true);
        const ok = await changePassword(user.id, oldPassword, newPassword);
        setPasswordLoading(false);
        if (ok) {
            setPasswordChanged(true);
            setOldPassword('');
            setNewPassword('');
            setConfirmPassword('');
        } else {
            setPasswordError(t('settings.password_error'));
        }
    }

    return (
        <SettingsLayout>
            <div class="max-w-2xl mx-auto px-4 py-8">
                <h1 class="text-xl font-medium text-gray-700 dark:text-gray-200 mb-6">{t('settings.account')}</h1>

                <div class="flex flex-col gap-6">
                    {/* Profile form */}
                    <div class={CARD_CLASS}>
                        <h2 class="text-base font-semibold text-gray-700 dark:text-gray-200 mb-5">
                            {t('settings.profile_heading')}
                        </h2>
                        <form onSubmit={handleProfileSubmit} class="flex flex-col gap-4">
                            <div class="grid grid-cols-2 gap-4">
                                <FormField id="firstName" label={t('settings.first_name')}>
                                    <FormInput
                                        id="firstName"
                                        type="text"
                                        required
                                        value={firstName}
                                        onInput={(e) => setFirstName((e.target as HTMLInputElement).value)}
                                    />
                                </FormField>
                                <FormField id="lastName" label={t('settings.last_name')}>
                                    <FormInput
                                        id="lastName"
                                        type="text"
                                        required
                                        value={lastName}
                                        onInput={(e) => setLastName((e.target as HTMLInputElement).value)}
                                    />
                                </FormField>
                            </div>

                            <FormField id="email" label={t('settings.email')}>
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

                            <div class="flex items-center justify-between pt-1">
                                <div>
                                    {profileSaved && (
                                        <p class="text-sm text-green-600 dark:text-green-400">{t('settings.saved')}</p>
                                    )}
                                    {profileError && (
                                        <p class="text-sm text-red-500 dark:text-red-400">{t('settings.save_error')}</p>
                                    )}
                                </div>
                                <Button type="submit" disabled={profileLoading}>
                                    <span class="flex items-center gap-1.5">
                                        <svg class="w-4 h-4" viewBox="0 0 24 24" fill="currentColor">
                                            <path d="M17 3H5c-1.11 0-2 .9-2 2v14c0 1.1.89 2 2 2h14c1.1 0 2-.9 2-2V7l-4-4zm-5 16c-1.66 0-3-1.34-3-3s1.34-3 3-3 3 1.34 3 3-1.34 3-3 3zm3-10H5V5h10v4z" />
                                        </svg>
                                        {t('settings.save')}
                                    </span>
                                </Button>
                            </div>
                        </form>
                    </div>

                    {/* Change password form */}
                    <div class={CARD_CLASS}>
                        <h2 class="text-base font-semibold text-gray-700 dark:text-gray-200 mb-5">
                            {t('settings.password_heading')}
                        </h2>
                        <form onSubmit={handlePasswordSubmit} class="flex flex-col gap-4">
                            <FormField id="oldPassword" label={t('settings.current_password')}>
                                <FormInput
                                    id="oldPassword"
                                    type="password"
                                    required
                                    placeholder="••••••••"
                                    value={oldPassword}
                                    onInput={(e) => setOldPassword((e.target as HTMLInputElement).value)}
                                />
                            </FormField>
                            <FormField id="newPassword" label={t('settings.new_password')}>
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

                            <div class="flex items-center justify-between pt-1">
                                <div>
                                    {passwordChanged && (
                                        <p class="text-sm text-green-600 dark:text-green-400">
                                            {t('settings.password_changed')}
                                        </p>
                                    )}
                                    {passwordError && (
                                        <p class="text-sm text-red-500 dark:text-red-400">{passwordError}</p>
                                    )}
                                </div>
                                <Button type="submit" disabled={passwordLoading}>
                                    <span class="flex items-center gap-1.5">
                                        <svg class="w-4 h-4" viewBox="0 0 24 24" fill="currentColor">
                                            <path d="M18 8h-1V6c0-2.76-2.24-5-5-5S7 3.24 7 6v2H6c-1.1 0-2 .9-2 2v10c0 1.1.9 2 2 2h12c1.1 0 2-.9 2-2V10c0-1.1-.9-2-2-2zm-6 9c-1.1 0-2-.9-2-2s.9-2 2-2 2 .9 2 2-.9 2-2 2zm3.1-9H8.9V6c0-1.71 1.39-3.1 3.1-3.1 1.71 0 3.1 1.39 3.1 3.1v2z" />
                                        </svg>
                                        {t('settings.change_password')}
                                    </span>
                                </Button>
                            </div>
                        </form>
                    </div>
                </div>
            </div>
        </SettingsLayout>
    );
}
