/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import {
    type Pagination,
    type Report,
    type User,
    type UserCreateBody,
    type UserIndexResponse,
    type UserUpdateBody,
} from '../../src-gen/api.ts';
import { authFetch } from './auth.service.ts';

export async function listUsers(
    page = 1,
    _query?: string,
    signal?: AbortSignal,
): Promise<{ data: User[]; pagination: Pagination }> {
    const res = await authFetch(`/api/users?page=${page}`, { signal });
    if (!res.ok) return { data: [], pagination: { page, limit: 20, total: 0 } };
    const result: UserIndexResponse = await res.json();
    return result;
}

export async function createUser(params: UserCreateBody): Promise<{ data: User | null; report: Report | null }> {
    const res = await authFetch(`/api/users`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
            firstName: params.firstName,
            lastName: params.lastName,
            email: params.email,
            password: params.password,
            role: params.role,
        }),
    });
    if (!res.ok) return { data: null, report: await res.json().catch(() => null) };
    return { data: await res.json(), report: null };
}

export async function deleteUser(id: string): Promise<boolean> {
    const res = await authFetch(`/api/users/${id}`, { method: 'DELETE' });
    return res.ok;
}

export async function updateUser(
    id: string,
    params: UserUpdateBody,
): Promise<{ data: User | null; report: Report | null }> {
    const body: Record<string, string> = {
        firstName: params.firstName,
        lastName: params.lastName,
        email: params.email,
        theme: params.theme,
        language: params.language,
        role: params.role,
    };
    if (params.password) body.password = params.password;
    const res = await authFetch(`/api/users/${id}`, {
        method: 'PUT',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(body),
    });
    if (!res.ok) return { data: null, report: await res.json().catch(() => null) };
    return { data: await res.json(), report: null };
}

export async function changePassword(
    id: string,
    oldPassword: string,
    newPassword: string,
): Promise<{ ok: boolean; report: Report | null }> {
    const res = await authFetch(`/api/users/${id}/change-password`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ oldPassword, newPassword }),
    });
    if (!res.ok) return { ok: false, report: await res.json().catch(() => null) };
    return { ok: true, report: null };
}
