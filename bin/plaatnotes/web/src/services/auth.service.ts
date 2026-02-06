/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import { signal } from '@preact/signals';
import { API_URL } from '../consts.ts';
import type { AuthValidateResponse, LoginResponse, Session, User } from '../../src-gen/api.ts';

export const $authToken = signal<string | null>(null);
export const $authSession = signal<Session | null>(null);
export const $authUser = signal<User | null>(null);
export const $isLoading = signal<boolean>(true);

export class AuthService {
    static instance?: AuthService;

    static getInstance(): AuthService {
        if (AuthService.instance === undefined) {
            AuthService.instance = new AuthService();
        }
        return AuthService.instance;
    }

    async login(email: string, password: string): Promise<boolean> {
        try {
            const res = await fetch(`${API_URL}/auth/login`, {
                method: 'POST',
                body: new URLSearchParams({ email, password }),
            });
            if (res.status !== 200) {
                return false;
            }
            const { token } = (await res.json()) as LoginResponse;

            // Store token
            localStorage.setItem('authToken', token);
            await this.updateAuth();
            return true;
        } catch {
            return false;
        }
    }

    async updateAuth(): Promise<void> {
        const token = localStorage.getItem('authToken');
        if (!token) {
            $authToken.value = null;
            $authSession.value = null;
            $authUser.value = null;
            $isLoading.value = false;
            return;
        }

        try {
            const res = await fetch(`${API_URL}/auth/validate`, {
                headers: {
                    Authorization: `Bearer ${token}`,
                },
            });
            if (res.status !== 200) {
                localStorage.removeItem('authToken');
                $authToken.value = null;
                $authSession.value = null;
                $authUser.value = null;
                $isLoading.value = false;
                return;
            }

            const { session, user } = (await res.json()) as AuthValidateResponse;
            $authToken.value = token;
            $authSession.value = session;
            $authUser.value = user;
        } catch {
            localStorage.removeItem('authToken');
            $authToken.value = null;
            $authSession.value = null;
            $authUser.value = null;
        }
        $isLoading.value = false;
    }

    async logout(): Promise<void> {
        try {
            await fetch(`${API_URL}/auth/logout`, {
                method: 'POST',
                headers: {
                    Authorization: `Bearer ${$authToken.value}`,
                },
            });
        } catch {
            // Ignore errors
        }

        localStorage.removeItem('authToken');
        $authToken.value = null;
        $authSession.value = null;
        $authUser.value = null;
    }
}
