/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import {
    Button,
    Card,
    CardTitle,
    ContentSaveIcon,
    Form,
    FormActions,
    FormField,
    FormFooter,
    FormInput,
    FormMessage,
    FormRow,
    FormSelect,
    IconText,
    LockIcon,
    Page,
    PageTitle,
} from 'plaatui';
import { useEffect, useState } from 'preact/hooks';
import { type Report, type UserTheme } from '../../../src-gen/api.ts';
import { SettingsLayout } from '../../components/settings-layout.tsx';
import { $authUser } from '../../services/auth.service.ts';
import { t } from '../../services/i18n.service.ts';
import { changePassword, updateUser } from '../../services/users.service.ts';
import '../../components/list.css';

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
            <Page size="narrow">
                <PageTitle>{t('settings.account')}</PageTitle>

                <div class="list is-large-gap">
                    {/* Profile form */}
                    <Card>
                        <CardTitle>{t('settings.profile_heading')}</CardTitle>
                        <Form onSubmit={handleProfileSubmit}>
                            <FormRow>
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
                            </FormRow>

                            <FormField id="email" label={t('settings.email')} error={profileReport?.['email']?.[0]}>
                                <FormInput
                                    id="email"
                                    type="email"
                                    required
                                    value={email}
                                    onInput={(e) => setEmail((e.target as HTMLInputElement).value)}
                                />
                            </FormField>

                            <FormRow>
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
                            </FormRow>

                            <FormFooter>
                                <div>
                                    <FormMessage type="success" message={profileSaved && t('settings.saved')} />
                                    <FormMessage type="error" message={profileError && t('form.errors_occurred')} />
                                </div>
                                <FormActions flush>
                                    <Button type="submit" disabled={profileLoading}>
                                        <IconText>
                                            <ContentSaveIcon class="is-sm" />
                                            {profileLoading ? t('settings.saving') : t('settings.save')}
                                        </IconText>
                                    </Button>
                                </FormActions>
                            </FormFooter>
                        </Form>
                    </Card>

                    {/* Change password form */}
                    <Card>
                        <CardTitle>{t('settings.password_heading')}</CardTitle>
                        <Form onSubmit={handlePasswordSubmit}>
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

                            <FormFooter>
                                <div>
                                    <FormMessage
                                        type="success"
                                        message={passwordChanged && t('settings.password_changed')}
                                    />
                                    <FormMessage type="error" message={passwordReport && t('form.errors_occurred')} />
                                </div>
                                <FormActions flush>
                                    <Button type="submit" disabled={passwordLoading}>
                                        <IconText>
                                            <LockIcon class="is-sm" />
                                            {passwordLoading
                                                ? t('settings.changing_password')
                                                : t('settings.change_password')}
                                        </IconText>
                                    </Button>
                                </FormActions>
                            </FormFooter>
                        </Form>
                    </Card>
                </div>
            </Page>
        </SettingsLayout>
    );
}
