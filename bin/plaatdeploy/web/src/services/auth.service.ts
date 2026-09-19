/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import { signal } from '@preact/signals';
import { type AuthValidateResponse, type LoginResponse, type User } from '../../src-gen/api.ts';

const TOKEN_KEY = 'plaatdeploy-token';
export const $authUser = signal<User | null | undefined>(undefined);
export const $currentSessionId = signal<string | null>(null);

function clearAuth() {
    localStorage.removeItem(TOKEN_KEY);
    $authUser.value = null;
    $currentSessionId.value = null;
}

export async function authFetch(url: string, init: RequestInit = {}) {
    const headers = new Headers(init.headers);
    const token = localStorage.getItem(TOKEN_KEY);
    if (token) headers.set('Authorization', `Bearer ${token}`);
    const response = await fetch(url, { ...init, headers });
    if (response.status === 401) clearAuth();
    return response;
}

async function validate(token: string) {
    const response = await fetch('/api/auth/validate', { headers: { Authorization: `Bearer ${token}` } });
    if (response.status === 401) return false;
    if (!response.ok) throw new Error(`Could not validate session (${response.status})`);
    const data: AuthValidateResponse = await response.json();
    $authUser.value = data.user;
    $currentSessionId.value = data.session.id;
    return true;
}

export async function initAuth() {
    const token = localStorage.getItem(TOKEN_KEY);
    try {
        if (!token || !(await validate(token))) clearAuth();
    } catch {
        $authUser.value = null;
        $currentSessionId.value = null;
    }
}

export async function login(email: string, password: string) {
    try {
        const response = await fetch('/api/auth/login', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ email, password }),
        });
        if (!response.ok) return response.status === 429 ? 'rate_limited' : 'error';
        const data: LoginResponse = await response.json();
        if (!(await validate(data.token))) {
            clearAuth();
            return 'error';
        }
        localStorage.setItem(TOKEN_KEY, data.token);
        return 'success';
    } catch {
        return 'error';
    }
}

export async function logout() {
    try {
        await authFetch('/api/auth/logout', { method: 'POST' });
    } finally {
        clearAuth();
    }
}
