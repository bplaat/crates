/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import { signal } from '@preact/signals';
import { type User, type LoginResponse } from '../src-gen/api.ts';
import { API_URL } from './consts.ts';

export const $token = signal<string | null>(null);
export const $user = signal<User | null>(null);
export const $loading = signal(false);

const TOKEN_STORAGE_KEY = 'plaatnotes_token';

export function initAuth() {
    const token = localStorage.getItem(TOKEN_STORAGE_KEY);
    if (token) {
        $token.value = token;
    }
}

export function saveToken(token: string) {
    $token.value = token;
    localStorage.setItem(TOKEN_STORAGE_KEY, token);
}

export function clearAuth() {
    $token.value = null;
    $user.value = null;
    localStorage.removeItem(TOKEN_STORAGE_KEY);
}

export async function login(email: string, password: string): Promise<boolean> {
    $loading.value = true;
    try {
        const res = await fetch(`${API_URL}/auth/login`, {
            method: 'POST',
            body: new URLSearchParams({ email, password }),
        });

        if (res.ok) {
            const { token, userId }: LoginResponse = await res.json();
            saveToken(token);
            return true;
        }
        return false;
    } finally {
        $loading.value = false;
    }
}

export async function logout() {
    $loading.value = true;
    try {
        await fetch(`${API_URL}/auth/logout`, {
            method: 'POST',
            headers: { Authorization: `Bearer ${$token.value}` },
        });
    } finally {
        clearAuth();
        $loading.value = false;
    }
}

export function getAuthHeaders() {
    if (!$token.value) return {};
    return { Authorization: `Bearer ${$token.value}` };
}
