/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import { type Session, type SessionIndexResponse } from '../../src-gen/api.ts';
import { API_URL } from '../consts.ts';
import { $authUser, authFetch } from './auth.service.ts';

export async function listSessions(): Promise<Session[]> {
    const userId = $authUser.value!.id;
    const res = await authFetch(`${API_URL}/users/${userId}/sessions/active`);
    const { data }: SessionIndexResponse = await res.json();
    return data;
}

export async function revokeSession(id: string): Promise<boolean> {
    const res = await authFetch(`${API_URL}/sessions/${id}`, { method: 'DELETE' });
    return res.ok;
}
