/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import { type Pagination, type Session, type SessionIndexResponse } from '../../src-gen/api.ts';
import { API_URL } from '../consts.ts';
import { $authUser, authFetch } from './auth.service.ts';

export async function listSessions(page = 1): Promise<{ data: Session[]; pagination: Pagination }> {
    const userId = $authUser.value!.id;
    const res = await authFetch(`${API_URL}/users/${userId}/sessions/active?page=${page}`);
    if (!res.ok) return { data: [], pagination: { page, limit: 20, total: 0 } };
    const result: SessionIndexResponse = await res.json();
    return result;
}

export async function revokeSession(id: string): Promise<boolean> {
    const res = await authFetch(`${API_URL}/sessions/${id}`, { method: 'DELETE' });
    return res.ok;
}
