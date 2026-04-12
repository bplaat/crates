/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import { signal } from '@preact/signals';
import type { AuthValidateResponse, LoginResponse, User } from '../src-gen/api.ts';

const API_URL = '/api';
const TOKEN_KEY = 'token';

export const $authUser = signal<User | null | undefined>(undefined);
export const $currentSessionId = signal<string | null>(null);

export function getToken(): string | null {
    return localStorage.getItem(TOKEN_KEY);
}

export async function initAuth() {
    const token = getToken();
    if (!token) {
        $authUser.value = null;
        $currentSessionId.value = null;
        return;
    }
    const res = await fetch(`${API_URL}/auth/validate`, {
        headers: { Authorization: `Bearer ${token}` },
    });
    if (!res.ok) {
        localStorage.removeItem(TOKEN_KEY);
        $authUser.value = null;
        $currentSessionId.value = null;
        return;
    }
    const { user, session }: AuthValidateResponse = await res.json();
    $authUser.value = user;
    $currentSessionId.value = session.id;
}

export async function login(email: string, password: string): Promise<'success' | 'error' | 'rate_limited'> {
    const res = await fetch(`${API_URL}/auth/login`, {
        method: 'POST',
        body: new URLSearchParams({ email, password }),
    });
    if (res.status === 429) return 'rate_limited';
    if (!res.ok) return 'error';
    const { token }: LoginResponse = await res.json();
    localStorage.setItem(TOKEN_KEY, token);
    await initAuth();
    return 'success';
}

export async function logout() {
    const token = getToken();
    if (token) {
        await fetch(`${API_URL}/auth/logout`, {
            method: 'POST',
            headers: { Authorization: `Bearer ${token}` },
        });
        localStorage.removeItem(TOKEN_KEY);
    }
    $authUser.value = null;
    $currentSessionId.value = null;
}

export function authFetch(url: string, init: RequestInit = {}): Promise<Response> {
    const token = getToken();
    const headers = new Headers(init.headers);
    if (token) headers.set('Authorization', `Bearer ${token}`);
    return fetch(url, { ...init, headers });
}

export { API_URL };
