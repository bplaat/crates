/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import { API_URL } from '../consts.ts';
import type { User } from '../../src-gen/api.ts';
import { $authToken, $authUser } from './auth.service.ts';

export class UserService {
    static instance?: UserService;

    static getInstance(): UserService {
        if (UserService.instance === undefined) {
            UserService.instance = new UserService();
        }
        return UserService.instance;
    }

    async updateUser(userId: string, firstName: string, lastName: string, email: string, theme?: string): Promise<boolean> {
        try {
            const params = new URLSearchParams({ firstName, lastName, email });
            if (theme) {
                params.append('theme', theme);
            }
            const res = await fetch(`${API_URL}/users/${userId}`, {
                method: 'PUT',
                headers: {
                    Authorization: `Bearer ${$authToken.value}`,
                },
                body: params,
            });
            if (res.status !== 200) {
                return false;
            }
            const user = (await res.json()) as User;
            $authUser.value = user;
            return true;
        } catch {
            return false;
        }
    }

    async changePassword(userId: string, oldPassword: string, newPassword: string): Promise<boolean> {
        try {
            const res = await fetch(`${API_URL}/users/${userId}/change-password`, {
                method: 'POST',
                headers: {
                    Authorization: `Bearer ${$authToken.value}`,
                },
                body: new URLSearchParams({ oldPassword, newPassword }),
            });
            return res.status === 200;
        } catch {
            return false;
        }
    }

    async getAllUsers(): Promise<User[]> {
        try {
            const res = await fetch(`${API_URL}/users`, {
                headers: { Authorization: `Bearer ${$authToken.value}` },
            });
            if (res.status !== 200) {
                return [];
            }
            const data = await res.json();
            return data.data || [];
        } catch {
            return [];
        }
    }

    async createUser(
        firstName: string,
        lastName: string,
        email: string,
        password: string,
        role: string,
    ): Promise<User | null> {
        try {
            const res = await fetch(`${API_URL}/users`, {
                method: 'POST',
                headers: { Authorization: `Bearer ${$authToken.value}` },
                body: new URLSearchParams({ firstName, lastName, email, password, role }),
            });

            if (res.status !== 200) {
                return null;
            }
            return (await res.json()) as User;
        } catch {
            return null;
        }
    }

    async deleteUser(userId: string): Promise<boolean> {
        try {
            const res = await fetch(`${API_URL}/users/${userId}`, {
                method: 'DELETE',
                headers: { Authorization: `Bearer ${$authToken.value}` },
            });
            return res.status === 200;
        } catch {
            return false;
        }
    }
}
