/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import type { SessionIndexResponse } from '../src-gen/api.ts';
import { API_URL, authFetch } from './auth.ts';

export async function listSessions(): Promise<SessionIndexResponse> {
    const response = await authFetch(`${API_URL}/sessions`);
    return (await response.json()) as SessionIndexResponse;
}

export async function revokeSession(sessionId: string): Promise<boolean> {
    const response = await authFetch(`${API_URL}/sessions/${sessionId}`, {
        method: 'DELETE',
    });
    return response.ok;
}
