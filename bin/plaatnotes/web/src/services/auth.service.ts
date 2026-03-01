/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import { signal } from '@preact/signals';
import { type AuthValidateResponse, type User } from '../../src-gen/api.ts';
import { API_URL } from '../consts.ts';

const TOKEN_KEY = 'token';

// undefined = loading, null = not authenticated, User = authenticated
export const $authUser = signal<User | null | undefined>(undefined);

export function getToken(): string | null {
    return localStorage.getItem(TOKEN_KEY);
}

export async function initAuth() {
    const token = getToken();
    if (!token) {
        $authUser.value = null;
        return;
    }
    const res = await fetch(`${API_URL}/auth/validate`, {
        headers: { Authorization: `Bearer ${token}` },
    });
    if (res.ok) {
        const { user }: AuthValidateResponse = await res.json();
        $authUser.value = user;
    } else {
        localStorage.removeItem(TOKEN_KEY);
        $authUser.value = null;
    }
}

export async function login(email: string, password: string): Promise<boolean> {
    const res = await fetch(`${API_URL}/auth/login`, {
        method: 'POST',
        body: new URLSearchParams({ email, password }),
    });
    if (!res.ok) return false;
    const { token }: { token: string } = await res.json();
    localStorage.setItem(TOKEN_KEY, token);
    const validate = await fetch(`${API_URL}/auth/validate`, {
        headers: { Authorization: `Bearer ${token}` },
    });
    if (validate.ok) {
        const { user }: AuthValidateResponse = await validate.json();
        $authUser.value = user;
    }
    return true;
}

export async function logout() {
    const token = getToken();
    if (token) {
        await fetch(`${API_URL}/auth/logout`, {
            method: 'POST',
            headers: { Authorization: `Bearer ${token}` },
        });
    }
    localStorage.removeItem(TOKEN_KEY);
    $authUser.value = null;
}

export function authFetch(url: string, init: RequestInit = {}): Promise<Response> {
    const token = getToken();
    const headers = new Headers(init.headers);
    if (token) headers.set('Authorization', `Bearer ${token}`);
    return fetch(url, { ...init, headers });
}
