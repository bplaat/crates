/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import { useEffect, useState } from 'preact/hooks';
import '../../components/toolbar.css';
import { useLocation } from 'wouter-preact';
import { type Report, type User, type UserRole, type UserUpdateBody } from '../../../src-gen/api.ts';
import { AdminLayout } from '../../components/admin-layout.tsx';
import { Button, SecondaryButton, SmallIconButton } from 'plaatui';
import { $authUser, loginAsUser } from '../../services/auth.service.ts';
import { Card } from 'plaatui';
import { ConfirmDialog, Dialog } from 'plaatui';
import { FormActions, FormField, FormMessage } from 'plaatui';
import { FormInput, FormSelect } from 'plaatui';
import { formatDate, t } from '../../services/i18n.service.ts';
import { Avatar, Badge, Icon, LoadingText, Table } from 'plaatui';
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
    const [confirmLoginAs, setConfirmLoginAs] = useState<User | null>(null);
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

    async function doLoginAs() {
        if (!confirmLoginAs) return;
        const ok = await loginAsUser(confirmLoginAs.id);
        setConfirmLoginAs(null);
        if (ok) navigate('/');
    }

    const isCreate = dialog?.kind === 'create';

    return (
        <AdminLayout>
            <div class="page is-wide">
                <div class="toolbar">
                    <h1 class="page-title">{t('admin.users.heading')}</h1>
                    <Button onClick={openCreate}>
                        <span class="icon-text">
                            <Icon type="plus" class="is-sm" />
                            {t('admin.users.create_user')}
                        </span>
                    </Button>
                </div>

                <Card class="is-clipped" padded={false}>
                    {loading && users.length === 0 && <LoadingText padded>{t('admin.users.loading')}</LoadingText>}
                    {!loading && users.length === 0 && <LoadingText padded>{t('admin.users.empty')}</LoadingText>}
                    {users.length > 0 && (
                        <Table>
                            <thead>
                                <tr>
                                    <th>{t('admin.users.col_name')}</th>
                                    <th class="col-hide-md">{t('admin.users.col_email')}</th>
                                    <th class="col-hide-sm">{t('admin.users.col_role')}</th>
                                    <th class="col-hide-lg">{t('admin.users.col_created')}</th>
                                    <th class="cell-actions">{t('admin.users.col_actions')}</th>
                                </tr>
                            </thead>
                            <tbody>
                                {users.map((user) => (
                                    <tr key={user.id}>
                                        <td>
                                            <div class="cell-name">
                                                <Avatar>
                                                    {user.firstName[0].toUpperCase()}
                                                    {lastNameInitial(user.lastName)}
                                                </Avatar>
                                                <span class="cell-name-text">
                                                    {user.firstName} {user.lastName}
                                                </span>
                                            </div>
                                        </td>
                                        <td class="col-hide-md has-text-muted">{user.email}</td>
                                        <td class="col-hide-sm">
                                            <Badge accent={user.role === 'admin'}>
                                                {user.role === 'admin'
                                                    ? t('admin.users.role_admin')
                                                    : t('admin.users.role_normal')}
                                            </Badge>
                                        </td>
                                        <td class="col-hide-lg has-text-subtle">{formatDate(user.createdAt)}</td>
                                        <td>
                                            <div class="table-actions">
                                                {user.id !== authUser.id && (
                                                    <SmallIconButton
                                                        onClick={() => setConfirmLoginAs(user)}
                                                        title={t('admin.users.login_as')}
                                                    >
                                                        <Icon type="login" class="is-sm" />
                                                    </SmallIconButton>
                                                )}
                                                <SmallIconButton
                                                    onClick={() => openEdit(user)}
                                                    title={t('admin.users.edit_user')}
                                                >
                                                    <Icon type="pencil" class="is-sm" />
                                                </SmallIconButton>
                                                <SmallIconButton
                                                    onClick={() => handleDelete(user)}
                                                    title={t('admin.users.delete_user')}
                                                    class="hover-danger"
                                                >
                                                    <Icon type="delete-outline" class="is-sm" />
                                                </SmallIconButton>
                                            </div>
                                        </td>
                                    </tr>
                                ))}
                            </tbody>
                        </Table>
                    )}

                    {hasMore && <div ref={sentinelRef} class="sentinel" />}
                    {loading && users.length > 0 && <LoadingText>{t('admin.users.loading')}</LoadingText>}
                </Card>
            </div>

            {dialog && (
                <Dialog
                    title={isCreate ? t('admin.users.create_user') : t('admin.users.edit_user')}
                    onClose={closeDialog}
                >
                    <form onSubmit={handleSubmit} class="form">
                        <div class="field-row">
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

                        <FormMessage type="error" message={report && t('form.errors_occurred')} />
                        <FormActions class="is-flush">
                            <SecondaryButton type="button" onClick={closeDialog}>
                                {t('dialog.cancel')}
                            </SecondaryButton>
                            <Button type="submit" disabled={submitting}>
                                <span class="icon-text">
                                    {isCreate ? (
                                        <Icon type="plus" class="is-sm" />
                                    ) : (
                                        <Icon type="content-save" class="is-sm" />
                                    )}
                                    {submitting
                                        ? isCreate
                                            ? t('admin.users.creating')
                                            : t('admin.users.saving')
                                        : isCreate
                                          ? t('admin.users.create')
                                          : t('admin.users.save')}
                                </span>
                            </Button>
                        </FormActions>
                    </form>
                </Dialog>
            )}

            {confirmDelete && (
                <ConfirmDialog
                    title={t('admin.users.delete_user')}
                    message={t('admin.users.confirm_delete')}
                    confirmLabel={t('admin.users.delete')}
                    cancelLabel={t('dialog.cancel')}
                    confirmText={confirmDelete.email}
                    typeToConfirmLabel={(value) => t('dialog.type_to_confirm', value)}
                    onConfirm={doDelete}
                    onClose={() => setConfirmDelete(null)}
                />
            )}

            {confirmLoginAs && (
                <ConfirmDialog
                    title={t('admin.users.login_as')}
                    cancelLabel={t('dialog.cancel')}
                    message={t(
                        'admin.users.confirm_login_as',
                        `${confirmLoginAs.firstName} ${confirmLoginAs.lastName}`,
                    )}
                    confirmLabel={t('admin.users.login_as')}
                    danger={false}
                    icon={<Icon type="login" class="is-sm" />}
                    onConfirm={doLoginAs}
                    onClose={() => setConfirmLoginAs(null)}
                />
            )}
        </AdminLayout>
    );
}
