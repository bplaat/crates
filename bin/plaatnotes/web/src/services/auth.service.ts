/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import { signal } from '@preact/signals';
import { type AuthValidateResponse, type LoginResponse, type User } from '../../src-gen/api.ts';
import { API_URL } from '../consts.ts';

const TOKEN_KEY = 'token';

// undefined = loading, null = not authenticated, User = authenticated
export const $authUser = signal<User | null | undefined>(undefined);
export const $currentSessionId = signal<string | null>(null);

export function getToken(): string | null {
    return localStorage.getItem(TOKEN_KEY);
}

async function applyValidate(token: string): Promise<boolean> {
    const res = await fetch(`${API_URL}/auth/validate`, {
        headers: { Authorization: `Bearer ${token}` },
    });
    if (!res.ok) return false;
    const { user, session }: AuthValidateResponse = await res.json();
    $authUser.value = user;
    $currentSessionId.value = session.id;
    return true;
}

export async function initAuth() {
    const token = getToken();
    if (!token) {
        $authUser.value = null;
        return;
    }
    const ok = await applyValidate(token);
    if (!ok) {
        localStorage.removeItem(TOKEN_KEY);
        $authUser.value = null;
    }
}

export function authFetch(url: string, init: RequestInit = {}): Promise<Response> {
    const token = getToken();
    const headers = new Headers(init.headers);
    if (token) headers.set('Authorization', `Bearer ${token}`);
    return fetch(url, { ...init, headers });
}

export async function login(email: string, password: string): Promise<boolean> {
    const res = await fetch(`${API_URL}/auth/login`, {
        method: 'POST',
        body: new URLSearchParams({ email, password }),
    });
    if (!res.ok) return false;
    const { token }: LoginResponse = await res.json();
    localStorage.setItem(TOKEN_KEY, token);
    await applyValidate(token);
    return true;
}

export async function logout() {
    await authFetch(`${API_URL}/auth/logout`, { method: 'POST' });
    localStorage.removeItem(TOKEN_KEY);
    $authUser.value = null;
    $currentSessionId.value = null;
}
