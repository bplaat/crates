/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import { route } from 'preact-router';
import { useEffect, useState } from 'preact/hooks';
import { type UserTheme } from '../../src-gen/api.ts';
import { Navbar } from '../components/navbar.tsx';
import { $authUser } from '../services/auth.service.ts';
import { t } from '../services/i18n.service.ts';
import { changePassword, updateUser } from '../services/users.service.ts';

const INPUT_CLASS =
    'w-full px-3 py-2 border border-gray-300 dark:border-zinc-600 rounded-lg text-sm bg-white dark:bg-zinc-700 text-gray-800 dark:text-gray-100 placeholder-gray-400 dark:placeholder-gray-500 focus:outline-none focus:ring-2 focus:ring-yellow-400 dark:focus:ring-yellow-500/50 focus:border-transparent';

const LABEL_CLASS = 'text-sm font-medium text-gray-700 dark:text-gray-300';

const CARD_CLASS = 'bg-white dark:bg-zinc-800 rounded-2xl border border-gray-200 dark:border-zinc-700 shadow-sm p-6';

export function Settings() {
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
        <div class="min-h-screen bg-gray-50 dark:bg-zinc-900">
            <Navbar />
            <main class="max-w-2xl mx-auto px-4 py-8">
                <div class="flex items-center gap-3 mb-6">
                    <button
                        onClick={() => route('/')}
                        class="p-2 rounded-full hover:bg-gray-200 dark:hover:bg-zinc-700 text-gray-500 dark:text-gray-400 transition-colors cursor-pointer"
                        title={t('notes_create.back')}
                    >
                        <svg class="w-5 h-5" viewBox="0 0 24 24" fill="currentColor">
                            <path d="M20 11H7.83l5.59-5.59L12 4l-8 8 8 8 1.41-1.41L7.83 13H20v-2z" />
                        </svg>
                    </button>
                    <h1 class="text-xl font-medium text-gray-700 dark:text-gray-200">{t('page.settings')}</h1>
                </div>

                <div class="flex flex-col gap-6">
                    {/* Profile form */}
                    <div class={CARD_CLASS}>
                        <h2 class="text-base font-semibold text-gray-700 dark:text-gray-200 mb-5">{t('settings.profile_heading')}</h2>
                        <form onSubmit={handleProfileSubmit} class="flex flex-col gap-4">
                            <div class="grid grid-cols-2 gap-4">
                                <div class="flex flex-col gap-1">
                                    <label for="firstName" class={LABEL_CLASS}>{t('settings.first_name')}</label>
                                    <input
                                        id="firstName"
                                        class={INPUT_CLASS}
                                        type="text"
                                        required
                                        value={firstName}
                                        onInput={(e) => setFirstName((e.target as HTMLInputElement).value)}
                                    />
                                </div>
                                <div class="flex flex-col gap-1">
                                    <label for="lastName" class={LABEL_CLASS}>{t('settings.last_name')}</label>
                                    <input
                                        id="lastName"
                                        class={INPUT_CLASS}
                                        type="text"
                                        required
                                        value={lastName}
                                        onInput={(e) => setLastName((e.target as HTMLInputElement).value)}
                                    />
                                </div>
                            </div>

                            <div class="flex flex-col gap-1">
                                <label for="email" class={LABEL_CLASS}>{t('settings.email')}</label>
                                <input
                                    id="email"
                                    class={INPUT_CLASS}
                                    type="email"
                                    required
                                    value={email}
                                    onInput={(e) => setEmail((e.target as HTMLInputElement).value)}
                                />
                            </div>

                            <div class="grid grid-cols-2 gap-4">
                                <div class="flex flex-col gap-1">
                                    <label for="theme" class={LABEL_CLASS}>{t('settings.theme')}</label>
                                    <select
                                        id="theme"
                                        class={INPUT_CLASS}
                                        value={theme}
                                        onChange={(e) => setTheme((e.target as HTMLSelectElement).value as UserTheme)}
                                    >
                                        <option value="system">{t('settings.theme_system')}</option>
                                        <option value="light">{t('settings.theme_light')}</option>
                                        <option value="dark">{t('settings.theme_dark')}</option>
                                    </select>
                                </div>
                                <div class="flex flex-col gap-1">
                                    <label for="language" class={LABEL_CLASS}>{t('settings.language')}</label>
                                    <select
                                        id="language"
                                        class={INPUT_CLASS}
                                        value={language}
                                        onChange={(e) => setLanguage((e.target as HTMLSelectElement).value)}
                                    >
                                        <option value="en">{t('settings.language_en')}</option>
                                        <option value="nl">{t('settings.language_nl')}</option>
                                    </select>
                                </div>
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
                                <button
                                    type="submit"
                                    disabled={profileLoading}
                                    class="px-4 py-2 bg-yellow-400 hover:bg-yellow-500 dark:bg-yellow-900/40 dark:hover:bg-yellow-900/60 dark:text-yellow-400 disabled:opacity-60 text-white text-sm font-medium rounded-lg transition-colors cursor-pointer"
                                >
                                    {t('settings.save')}
                                </button>
                            </div>
                        </form>
                    </div>

                    {/* Change password form */}
                    <div class={CARD_CLASS}>
                        <h2 class="text-base font-semibold text-gray-700 dark:text-gray-200 mb-5">{t('settings.password_heading')}</h2>
                        <form onSubmit={handlePasswordSubmit} class="flex flex-col gap-4">
                            <div class="flex flex-col gap-1">
                                <label for="oldPassword" class={LABEL_CLASS}>{t('settings.current_password')}</label>
                                <input
                                    id="oldPassword"
                                    class={INPUT_CLASS}
                                    type="password"
                                    required
                                    placeholder="••••••••"
                                    value={oldPassword}
                                    onInput={(e) => setOldPassword((e.target as HTMLInputElement).value)}
                                />
                            </div>
                            <div class="flex flex-col gap-1">
                                <label for="newPassword" class={LABEL_CLASS}>{t('settings.new_password')}</label>
                                <input
                                    id="newPassword"
                                    class={INPUT_CLASS}
                                    type="password"
                                    required
                                    placeholder="••••••••"
                                    value={newPassword}
                                    onInput={(e) => setNewPassword((e.target as HTMLInputElement).value)}
                                />
                            </div>
                            <div class="flex flex-col gap-1">
                                <label for="confirmPassword" class={LABEL_CLASS}>{t('settings.confirm_password')}</label>
                                <input
                                    id="confirmPassword"
                                    class={INPUT_CLASS}
                                    type="password"
                                    required
                                    placeholder="••••••••"
                                    value={confirmPassword}
                                    onInput={(e) => setConfirmPassword((e.target as HTMLInputElement).value)}
                                />
                            </div>

                            <div class="flex items-center justify-between pt-1">
                                <div>
                                    {passwordChanged && (
                                        <p class="text-sm text-green-600 dark:text-green-400">{t('settings.password_changed')}</p>
                                    )}
                                    {passwordError && (
                                        <p class="text-sm text-red-500 dark:text-red-400">{passwordError}</p>
                                    )}
                                </div>
                                <button
                                    type="submit"
                                    disabled={passwordLoading}
                                    class="px-4 py-2 bg-yellow-400 hover:bg-yellow-500 dark:bg-yellow-900/40 dark:hover:bg-yellow-900/60 dark:text-yellow-400 disabled:opacity-60 text-white text-sm font-medium rounded-lg transition-colors cursor-pointer"
                                >
                                    {t('settings.change_password')}
                                </button>
                            </div>
                        </form>
                    </div>
                </div>
            </main>
        </div>
    );
}
