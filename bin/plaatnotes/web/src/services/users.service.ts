/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import { type User, type UserRole, type UserTheme } from '../../src-gen/api.ts';
import { API_URL } from '../consts.ts';
import { authFetch } from './auth.service.ts';

export async function updateUser(
    id: string,
    params: { firstName: string; lastName: string; email: string; theme: UserTheme; language: string; role: UserRole },
): Promise<User | null> {
    const form = new URLSearchParams({
        firstName: params.firstName,
        lastName: params.lastName,
        email: params.email,
        theme: params.theme,
        language: params.language,
        role: params.role,
    });
    const res = await authFetch(`${API_URL}/users/${id}`, { method: 'PUT', body: form });
    if (!res.ok) return null;
    return res.json();
}

export async function changePassword(id: string, oldPassword: string, newPassword: string): Promise<boolean> {
    const form = new URLSearchParams({ oldPassword, newPassword });
    const res = await authFetch(`${API_URL}/users/${id}/change-password`, { method: 'POST', body: form });
    return res.ok;
}
