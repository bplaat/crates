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
import { API_URL } from '../consts.ts';
import { authFetch } from './auth.service.ts';

export async function listUsers(page = 1): Promise<{ data: User[]; pagination: Pagination }> {
    const res = await authFetch(`${API_URL}/users?page=${page}`);
    const result: UserIndexResponse = await res.json();
    return result;
}

export async function createUser(params: UserCreateBody): Promise<{ data: User | null; report: Report | null }> {
    const form = new URLSearchParams({
        firstName: params.firstName,
        lastName: params.lastName,
        email: params.email,
        password: params.password,
        role: params.role,
    });
    const res = await authFetch(`${API_URL}/users`, { method: 'POST', body: form });
    if (!res.ok) return { data: null, report: await res.json().catch(() => null) };
    return { data: await res.json(), report: null };
}

export async function deleteUser(id: string): Promise<boolean> {
    const res = await authFetch(`${API_URL}/users/${id}`, { method: 'DELETE' });
    return res.ok;
}

export async function updateUser(
    id: string,
    params: UserUpdateBody,
): Promise<{ data: User | null; report: Report | null }> {
    const form = new URLSearchParams({
        firstName: params.firstName,
        lastName: params.lastName,
        email: params.email,
        theme: params.theme,
        language: params.language,
        role: params.role,
    });
    if (params.password) form.set('password', params.password);
    const res = await authFetch(`${API_URL}/users/${id}`, { method: 'PUT', body: form });
    if (!res.ok) return { data: null, report: await res.json().catch(() => null) };
    return { data: await res.json(), report: null };
}

export async function changePassword(
    id: string,
    oldPassword: string,
    newPassword: string,
): Promise<{ ok: boolean; report: Report | null }> {
    const form = new URLSearchParams({ oldPassword, newPassword });
    const res = await authFetch(`${API_URL}/users/${id}/change-password`, { method: 'POST', body: form });
    if (!res.ok) return { ok: false, report: await res.json().catch(() => null) };
    return { ok: true, report: null };
}
