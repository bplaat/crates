/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import type { Report, User } from '../src-gen/api.ts';
import { API_URL, authFetch } from './auth.ts';

export async function updateCurrentUser(
    userId: string,
    body: { firstName: string; lastName: string; email: string },
): Promise<{ data?: User; report?: Report }> {
    const response = await authFetch(`${API_URL}/users/${userId}`, {
        method: 'PUT',
        body: new URLSearchParams({
            first_name: body.firstName,
            last_name: body.lastName,
            email: body.email,
        }),
    });
    if (!response.ok) {
        return { report: (await response.json()) as Report };
    }
    return { data: (await response.json()) as User };
}

export async function changePassword(
    userId: string,
    oldPassword: string,
    newPassword: string,
): Promise<{ ok: boolean; report?: Report }> {
    const response = await authFetch(`${API_URL}/users/${userId}/change-password`, {
        method: 'POST',
        body: new URLSearchParams({
            old_password: oldPassword,
            new_password: newPassword,
        }),
    });
    if (!response.ok) {
        return { ok: false, report: (await response.json()) as Report };
    }
    return { ok: true };
}
