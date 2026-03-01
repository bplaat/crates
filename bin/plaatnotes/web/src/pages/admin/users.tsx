/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import { useEffect, useState } from 'preact/hooks';
import { type User, type UserRole } from '../../../src-gen/api.ts';
import { AdminLayout } from '../../components/admin-layout.tsx';
import { Dialog } from '../../components/dialog.tsx';
import { formatDate, t } from '../../services/i18n.service.ts';
import { createUser, deleteUser, listUsers, updateUser } from '../../services/users.service.ts';

const INPUT_CLASS =
    'w-full px-3 py-2 border border-gray-300 dark:border-zinc-600 rounded-lg text-sm bg-white dark:bg-zinc-700 text-gray-800 dark:text-gray-100 placeholder-gray-400 dark:placeholder-gray-500 focus:outline-none focus:ring-2 focus:ring-yellow-400 dark:focus:ring-yellow-500/50 focus:border-transparent';
const LABEL_CLASS = 'text-sm font-medium text-gray-700 dark:text-gray-300';
const BTN_PRIMARY =
    'px-4 py-2 bg-yellow-400 hover:bg-yellow-500 dark:bg-yellow-900/40 dark:hover:bg-yellow-900/60 dark:text-yellow-400 disabled:opacity-60 text-white text-sm font-medium rounded-lg transition-colors cursor-pointer';
const BTN_ICON =
    'p-1.5 rounded-lg text-gray-400 dark:text-gray-500 hover:bg-gray-100 dark:hover:bg-zinc-700 transition-colors cursor-pointer';

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
    const [users, setUsers] = useState<User[]>([]);
    const [loading, setLoading] = useState(true);
    const [dialog, setDialog] = useState<DialogMode | null>(null);
    const [form, setForm] = useState<UserFormState>(emptyForm());
    const [submitting, setSubmitting] = useState(false);

    // @ts-ignore
    useEffect(async () => {
        document.title = `PlaatNotes - ${t('admin.users_heading')}`;
        const data = await listUsers();
        setUsers(data);
        setLoading(false);
    }, []);

    function openCreate() {
        setForm(emptyForm());
        setDialog({ kind: 'create' });
    }

    function openEdit(user: User) {
        setForm(formFromUser(user));
        setDialog({ kind: 'edit', user });
    }

    function closeDialog() {
        setDialog(null);
        setSubmitting(false);
    }

    async function handleSubmit(e: SubmitEvent) {
        e.preventDefault();
        setSubmitting(true);
        if (dialog!.kind === 'create') {
            const created = await createUser(form);
            if (created) {
                setUsers((us) => [...us, created]);
                closeDialog();
            }
        } else {
            const target = (dialog as { kind: 'edit'; user: User }).user;
            const updated = await updateUser(target.id, {
                firstName: form.firstName,
                lastName: form.lastName,
                email: form.email,
                theme: target.theme,
                language: target.language,
                role: form.role,
            });
            if (updated) {
                setUsers((us) => us.map((u) => (u.id === updated.id ? updated : u)));
                closeDialog();
            }
        }
        setSubmitting(false);
    }

    async function handleDelete(user: User) {
        if (!confirm(t('admin.confirm_delete'))) return;
        const ok = await deleteUser(user.id);
        if (ok) setUsers((us) => us.filter((u) => u.id !== user.id));
    }

    const isCreate = dialog?.kind === 'create';

    return (
        <AdminLayout>
            <div class="px-4 py-8">
                <div class="flex items-center justify-between mb-6">
                    <h1 class="text-xl font-medium text-gray-700 dark:text-gray-200">{t('admin.users_heading')}</h1>
                    <button onClick={openCreate} class={BTN_PRIMARY}>
                        <span class="flex items-center gap-1.5">
                            <svg class="w-4 h-4" viewBox="0 0 24 24" fill="currentColor">
                                <path d="M19 13h-6v6h-2v-6H5v-2h6V5h2v6h6v2z" />
                            </svg>
                            {t('admin.create_user')}
                        </span>
                    </button>
                </div>

                <div class="bg-white dark:bg-zinc-800 rounded-2xl border border-gray-200 dark:border-zinc-700 shadow-sm overflow-hidden">
                    {loading && <p class="text-center text-gray-400 dark:text-gray-500 py-16">{t('admin.loading')}</p>}
                    {!loading && users.length === 0 && (
                        <p class="text-center text-gray-400 dark:text-gray-500 py-16">{t('admin.empty')}</p>
                    )}
                    {!loading && users.length > 0 && (
                        <table class="w-full text-sm">
                            <thead>
                                <tr class="border-b border-gray-100 dark:border-zinc-700 text-left text-xs font-semibold text-gray-500 dark:text-gray-400 uppercase tracking-wide">
                                    <th class="px-5 py-3">{t('admin.col_name')}</th>
                                    <th class="px-5 py-3 hidden md:table-cell">{t('admin.col_email')}</th>
                                    <th class="px-5 py-3 hidden sm:table-cell">{t('admin.col_role')}</th>
                                    <th class="px-5 py-3 hidden lg:table-cell">{t('admin.col_created')}</th>
                                    <th class="px-5 py-3 text-right">{t('admin.col_actions')}</th>
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
                                                    {user.firstName[0]}
                                                    {user.lastName[0]}
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
                                                {user.role === 'admin' ? t('admin.role_admin') : t('admin.role_normal')}
                                            </span>
                                        </td>
                                        <td class="px-5 py-3 hidden lg:table-cell text-gray-400 dark:text-gray-500">
                                            {formatDate(user.createdAt)}
                                        </td>
                                        <td class="px-5 py-3">
                                            <div class="flex items-center justify-end gap-1">
                                                <button
                                                    onClick={() => openEdit(user)}
                                                    title={t('admin.edit_user')}
                                                    class={BTN_ICON}
                                                >
                                                    <svg class="w-4 h-4" viewBox="0 0 24 24" fill="currentColor">
                                                        <path d="M3 17.25V21h3.75L17.81 9.94l-3.75-3.75L3 17.25zM20.71 7.04c.39-.39.39-1.02 0-1.41l-2.34-2.34c-.39-.39-1.02-.39-1.41 0l-1.83 1.83 3.75 3.75 1.83-1.83z" />
                                                    </svg>
                                                </button>
                                                <button
                                                    onClick={() => handleDelete(user)}
                                                    title={t('admin.confirm_delete')}
                                                    class={`${BTN_ICON} hover:text-red-500 dark:hover:text-red-400`}
                                                >
                                                    <svg class="w-4 h-4" viewBox="0 0 24 24" fill="currentColor">
                                                        <path d="M6 19c0 1.1.9 2 2 2h8c1.1 0 2-.9 2-2V7H6v12zM8 9h8v10H8V9zm7.5-5l-1-1h-5l-1 1H5v2h14V4h-3.5z" />
                                                    </svg>
                                                </button>
                                            </div>
                                        </td>
                                    </tr>
                                ))}
                            </tbody>
                        </table>
                    )}
                </div>
            </div>

            {dialog && (
                <Dialog title={isCreate ? t('admin.create_user') : t('admin.edit_user')} onClose={closeDialog}>
                    <form onSubmit={handleSubmit} class="flex flex-col gap-4">
                        <div class="grid grid-cols-2 gap-4">
                            <div class="flex flex-col gap-1">
                                <label for="firstName" class={LABEL_CLASS}>
                                    {t('admin.first_name')}
                                </label>
                                <input
                                    id="firstName"
                                    class={INPUT_CLASS}
                                    type="text"
                                    required
                                    value={form.firstName}
                                    onInput={(e) =>
                                        setForm({ ...form, firstName: (e.target as HTMLInputElement).value })
                                    }
                                />
                            </div>
                            <div class="flex flex-col gap-1">
                                <label for="lastName" class={LABEL_CLASS}>
                                    {t('admin.last_name')}
                                </label>
                                <input
                                    id="lastName"
                                    class={INPUT_CLASS}
                                    type="text"
                                    required
                                    value={form.lastName}
                                    onInput={(e) =>
                                        setForm({ ...form, lastName: (e.target as HTMLInputElement).value })
                                    }
                                />
                            </div>
                        </div>

                        <div class="flex flex-col gap-1">
                            <label for="email" class={LABEL_CLASS}>
                                {t('admin.email')}
                            </label>
                            <input
                                id="email"
                                class={INPUT_CLASS}
                                type="email"
                                required
                                value={form.email}
                                onInput={(e) => setForm({ ...form, email: (e.target as HTMLInputElement).value })}
                            />
                        </div>

                        {isCreate && (
                            <div class="flex flex-col gap-1">
                                <label for="password" class={LABEL_CLASS}>
                                    {t('admin.password')}
                                </label>
                                <input
                                    id="password"
                                    class={INPUT_CLASS}
                                    type="password"
                                    required
                                    placeholder="••••••••"
                                    value={form.password}
                                    onInput={(e) =>
                                        setForm({ ...form, password: (e.target as HTMLInputElement).value })
                                    }
                                />
                            </div>
                        )}

                        <div class="flex flex-col gap-1">
                            <label for="role" class={LABEL_CLASS}>
                                {t('admin.role')}
                            </label>
                            <select
                                id="role"
                                class={INPUT_CLASS}
                                value={form.role}
                                onChange={(e) =>
                                    setForm({ ...form, role: (e.target as HTMLSelectElement).value as UserRole })
                                }
                            >
                                <option value="normal">{t('admin.role_normal')}</option>
                                <option value="admin">{t('admin.role_admin')}</option>
                            </select>
                        </div>

                        <div class="flex justify-end pt-1">
                            <button type="submit" disabled={submitting} class={BTN_PRIMARY}>
                                {isCreate ? t('admin.create') : t('admin.save')}
                            </button>
                        </div>
                    </form>
                </Dialog>
            )}
        </AdminLayout>
    );
}
