/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import { useEffect, useState } from 'preact/hooks';
import { useLocation } from 'wouter-preact';
import { type Report, type User, type UserRole, type UserUpdateBody } from '../../../src-gen/api.ts';
import { AdminLayout } from '../../components/admin-layout.tsx';
import { Button, SmallIconButton } from '../../components/button.tsx';
import { $authUser } from '../../services/auth.service.ts';
import { Card } from '../../components/card.tsx';
import { ConfirmDialog, Dialog } from '../../components/dialog.tsx';
import { FormActions, FormField, FormMessage } from '../../components/form.tsx';
import { FormInput, FormSelect } from '../../components/input.tsx';
import { formatDate, t } from '../../services/i18n.service.ts';
import { ContentSaveIcon, DeleteOutlineIcon, PencilIcon, PlusIcon } from '../../components/icons.tsx';
import { lastNameInitial } from '../../utils.ts';
import { useInfiniteScroll } from '../../hooks/use-infinite-scroll.ts';
import { createUser, deleteUser, listUsers, updateUser } from '../../services/users.service.ts';

type DialogMode = { kind: 'create' } | { kind: 'edit'; user: User };

interface UserFormState {
    firstName: string;
    lastName: string;
    email: string;
    password: string;
    role: UserRole;
}

function emptyForm(): UserFormState {
    return { firstName: '', lastName: '', email: '', password: '', role: 'normal' };
}

function formFromUser(user: User): UserFormState {
    return { firstName: user.firstName, lastName: user.lastName, email: user.email, password: '', role: user.role };
}

export function AdminUsers() {
    const [, navigate] = useLocation();
    const authUser = $authUser.value;
    const { items: users, loading, hasMore, sentinelRef, setItems: setUsers } = useInfiniteScroll(listUsers);
    const [dialog, setDialog] = useState<DialogMode | null>(null);
    const [form, setForm] = useState<UserFormState>(emptyForm());
    const [submitting, setSubmitting] = useState(false);
    const [confirmDelete, setConfirmDelete] = useState<User | null>(null);
    const [report, setReport] = useState<Report | null>(null);

    useEffect(() => {
        document.title = `PlaatNotes - ${t('admin.users.heading')}`;
    }, []);

    useEffect(() => {
        if (authUser && authUser.role !== 'admin') navigate('/');
    }, [authUser]);

    if (!authUser || authUser.role !== 'admin') return null;

    function openCreate() {
        setForm(emptyForm());
        setReport(null);
        setDialog({ kind: 'create' });
    }

    function openEdit(user: User) {
        setForm(formFromUser(user));
        setReport(null);
        setDialog({ kind: 'edit', user });
    }

    function closeDialog() {
        setDialog(null);
        setReport(null);
        setSubmitting(false);
    }

    async function handleSubmit(e: SubmitEvent) {
        e.preventDefault();
        setSubmitting(true);
        setReport(null);
        if (dialog!.kind === 'create') {
            const { data: created, report: r } = await createUser(form);
            if (created) {
                setUsers((us) => [...us, created]);
                closeDialog();
            } else {
                setReport(r);
            }
        } else {
            const target = (dialog as { kind: 'edit'; user: User }).user;
            const { data: updated, report: r } = await updateUser(target.id, {
                firstName: form.firstName,
                lastName: form.lastName,
                email: form.email,
                password: form.password || undefined,
                theme: target.theme,
                language: target.language,
                role: form.role,
            } satisfies UserUpdateBody);
            if (updated) {
                setUsers((us) => us.map((u) => (u.id === updated.id ? updated : u)));
                closeDialog();
            } else {
                setReport(r);
            }
        }
        setSubmitting(false);
    }

    function handleDelete(user: User) {
        setConfirmDelete(user);
    }

    async function doDelete() {
        if (!confirmDelete) return;
        const ok = await deleteUser(confirmDelete.id);
        if (ok) setUsers((us) => us.filter((u) => u.id !== confirmDelete.id));
        setConfirmDelete(null);
    }

    const isCreate = dialog?.kind === 'create';

    return (
        <AdminLayout>
            <div class="px-4 py-8">
                <div class="flex items-center justify-between mb-6">
                    <h1 class="text-xl font-medium text-gray-700 dark:text-gray-200">{t('admin.users.heading')}</h1>
                    <Button onClick={openCreate}>
                        <span class="flex items-center gap-1.5">
                            <PlusIcon class="w-4 h-4" />
                            {t('admin.users.create_user')}
                        </span>
                    </Button>
                </div>

                <Card class="overflow-hidden">
                    {loading && users.length === 0 && (
                        <p class="text-center text-gray-400 dark:text-gray-500 py-16">{t('admin.users.loading')}</p>
                    )}
                    {!loading && users.length === 0 && (
                        <p class="text-center text-gray-400 dark:text-gray-500 py-16">{t('admin.users.empty')}</p>
                    )}
                    {users.length > 0 && (
                        <table class="w-full text-sm">
                            <thead>
                                <tr class="border-b border-gray-100 dark:border-zinc-700 text-left text-xs font-semibold text-gray-500 dark:text-gray-400 uppercase tracking-wide">
                                    <th class="px-5 py-3">{t('admin.users.col_name')}</th>
                                    <th class="px-5 py-3 hidden md:table-cell">{t('admin.users.col_email')}</th>
                                    <th class="px-5 py-3 hidden sm:table-cell">{t('admin.users.col_role')}</th>
                                    <th class="px-5 py-3 hidden lg:table-cell">{t('admin.users.col_created')}</th>
                                    <th class="px-5 py-3 text-right">{t('admin.users.col_actions')}</th>
                                </tr>
                            </thead>
                            <tbody>
                                {users.map((user) => (
                                    <tr
                                        key={user.id}
                                        class="border-b border-gray-50 dark:border-zinc-700/50 last:border-0 hover:bg-gray-50 dark:hover:bg-zinc-700/30 transition-colors"
                                    >
                                        <td class="px-5 py-3">
                                            <div class="flex items-center gap-3">
                                                <div class="w-8 h-8 rounded-full bg-yellow-400 dark:bg-yellow-900/40 text-white dark:text-yellow-400 font-semibold text-xs flex items-center justify-center shrink-0 select-none">
                                                    {user.firstName[0].toUpperCase()}
                                                    {lastNameInitial(user.lastName)}
                                                </div>
                                                <span class="font-medium text-gray-800 dark:text-gray-100">
                                                    {user.firstName} {user.lastName}
                                                </span>
                                            </div>
                                        </td>
                                        <td class="px-5 py-3 hidden md:table-cell text-gray-500 dark:text-gray-400">
                                            {user.email}
                                        </td>
                                        <td class="px-5 py-3 hidden sm:table-cell">
                                            <span
                                                class={`inline-flex items-center px-2 py-0.5 rounded-full text-xs font-medium ${
                                                    user.role === 'admin'
                                                        ? 'bg-yellow-100 dark:bg-yellow-900/40 text-yellow-700 dark:text-yellow-400'
                                                        : 'bg-gray-100 dark:bg-zinc-700 text-gray-600 dark:text-gray-400'
                                                }`}
                                            >
                                                {user.role === 'admin'
                                                    ? t('admin.users.role_admin')
                                                    : t('admin.users.role_normal')}
                                            </span>
                                        </td>
                                        <td class="px-5 py-3 hidden lg:table-cell text-gray-400 dark:text-gray-500">
                                            {formatDate(user.createdAt)}
                                        </td>
                                        <td class="px-5 py-3">
                                            <div class="flex items-center justify-end gap-1">
                                                <SmallIconButton
                                                    onClick={() => openEdit(user)}
                                                    title={t('admin.users.edit_user')}
                                                >
                                                    <PencilIcon class="w-4 h-4" />
                                                </SmallIconButton>
                                                <SmallIconButton
                                                    onClick={() => handleDelete(user)}
                                                    title={t('admin.users.delete_user')}
                                                    class="hover:text-red-500 dark:hover:text-red-400"
                                                >
                                                    <DeleteOutlineIcon class="w-4 h-4" />
                                                </SmallIconButton>
                                            </div>
                                        </td>
                                    </tr>
                                ))}
                            </tbody>
                        </table>
                    )}

                    {hasMore && <div ref={sentinelRef} class="h-1" />}
                    {loading && users.length > 0 && (
                        <p class="text-center text-gray-400 dark:text-gray-500 py-4">{t('admin.users.loading')}</p>
                    )}
                </Card>
            </div>

            {dialog && (
                <Dialog
                    title={isCreate ? t('admin.users.create_user') : t('admin.users.edit_user')}
                    onClose={closeDialog}
                >
                    <form onSubmit={handleSubmit} class="flex flex-col gap-4">
                        <div class="grid grid-cols-2 gap-4">
                            <FormField
                                id="firstName"
                                label={t('admin.users.first_name')}
                                error={report?.['first_name']?.[0]}
                            >
                                <FormInput
                                    id="firstName"
                                    type="text"
                                    required
                                    value={form.firstName}
                                    onInput={(e) =>
                                        setForm({ ...form, firstName: (e.target as HTMLInputElement).value })
                                    }
                                />
                            </FormField>
                            <FormField
                                id="lastName"
                                label={t('admin.users.last_name')}
                                error={report?.['last_name']?.[0]}
                            >
                                <FormInput
                                    id="lastName"
                                    type="text"
                                    required
                                    value={form.lastName}
                                    onInput={(e) =>
                                        setForm({ ...form, lastName: (e.target as HTMLInputElement).value })
                                    }
                                />
                            </FormField>
                        </div>

                        <FormField id="email" label={t('admin.users.email')} error={report?.['email']?.[0]}>
                            <FormInput
                                id="email"
                                type="email"
                                required
                                value={form.email}
                                onInput={(e) => setForm({ ...form, email: (e.target as HTMLInputElement).value })}
                            />
                        </FormField>

                        <FormField id="password" label={t('admin.users.password')} error={report?.['password']?.[0]}>
                            <FormInput
                                id="password"
                                type="password"
                                required={isCreate}
                                placeholder={!isCreate ? t('admin.users.password_keep') : ''}
                                value={form.password}
                                onInput={(e) => setForm({ ...form, password: (e.target as HTMLInputElement).value })}
                            />
                        </FormField>

                        <FormField id="role" label={t('admin.users.role')} error={report?.['role']?.[0]}>
                            <FormSelect
                                id="role"
                                value={form.role}
                                onChange={(e) =>
                                    setForm({ ...form, role: (e.target as HTMLSelectElement).value as UserRole })
                                }
                            >
                                <option value="normal">{t('admin.users.role_normal')}</option>
                                <option value="admin">{t('admin.users.role_admin')}</option>
                            </FormSelect>
                        </FormField>

                        <div class="flex flex-col gap-3 pt-1">
                            <FormMessage type="error" message={report && t('form.errors_occurred')} />
                            <FormActions class="pt-0">
                                <Button type="submit" disabled={submitting}>
                                    <span class="flex items-center gap-1.5">
                                        {isCreate ? <PlusIcon class="w-4 h-4" /> : <ContentSaveIcon class="w-4 h-4" />}
                                        {isCreate ? t('admin.users.create') : t('admin.users.save')}
                                    </span>
                                </Button>
                            </FormActions>
                        </div>
                    </form>
                </Dialog>
            )}

            {confirmDelete && (
                <ConfirmDialog
                    title={t('admin.users.delete_user')}
                    message={t('admin.users.confirm_delete')}
                    confirmLabel={t('admin.users.delete')}
                    onConfirm={doDelete}
                    onClose={() => setConfirmDelete(null)}
                />
            )}
        </AdminLayout>
    );
}
