/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import { type Page, expect, test } from '@playwright/test';

const API_URL = `http://localhost:${process.env.PLAYWRIGHT_PORT ?? '8080'}/api`;

async function authState(page: Page): Promise<{ token: string; userId: string; headers: Record<string, string> }> {
    if (!page.url().startsWith('http')) await page.goto('/');
    const token = await page.evaluate(() => localStorage.getItem('token') ?? '');
    const headers = { Authorization: `Bearer ${token}`, 'Content-Type': 'application/json' };
    const res = await page.request.get(`${API_URL}/auth/validate`, { headers });
    const { user } = await res.json();
    return { token, userId: user.id, headers };
}

test.describe('Admin - Users', () => {
    test('admin users page loads with correct title', async ({ page }) => {
        await page.goto('/admin/users');
        await expect(page).toHaveTitle(/PlaatNotes.*Users/);
        await expect(page.getByRole('heading', { name: 'Users' })).toBeVisible();
    });

    test('users table shows existing users', async ({ page }) => {
        await page.goto('/admin/users');
        // At minimum the two seeded test users should be visible
        await expect(page.getByRole('cell', { name: /Test/ }).first()).toBeVisible();
    });

    test('create user via dialog', async ({ page }) => {
        await page.goto('/admin/users');

        await page.getByRole('button', { name: 'Create user' }).click();

        const dialog = page.getByRole('dialog');
        await expect(dialog).toBeVisible();
        await expect(dialog.getByRole('heading', { name: 'Create user' })).toBeVisible();

        await dialog.getByLabel('First name').fill('New');
        await dialog.getByLabel('Last name').fill('Person');
        await dialog.getByLabel('Email').fill(`newperson-${Date.now()}@example.com`);
        await dialog.getByLabel('Password').fill('Password123!');

        const email = await dialog.getByLabel('Email').inputValue();

        await dialog.getByRole('button', { name: 'Create' }).click();

        // Dialog should close and user should appear in the table
        await expect(dialog).not.toBeVisible();
        await expect(page.getByText('New Person')).toBeVisible();

        // Cleanup: find and delete the created user
        const { headers } = await authState(page);
        const usersRes = await page.request.get(`${API_URL}/users`, { headers });
        const usersData = await usersRes.json();
        const created = usersData.data.find((u: { email: string }) => u.email === email);
        if (created) {
            await page.request.delete(`${API_URL}/users/${created.id}`, { headers });
        }
    });

    test('edit user via dialog', async ({ page }) => {
        // Create a user to edit
        const { headers } = await authState(page);
        const email = `editme-${Date.now()}@example.com`;
        const createRes = await page.request.post(`${API_URL}/users`, {
            headers,
            data: JSON.stringify({
                firstName: 'Edit',
                lastName: 'Me',
                email,
                password: 'Password123!',
                role: 'normal',
            }),
        });
        const created = await createRes.json();

        await page.goto('/admin/users');
        await expect(page.getByText('Edit Me')).toBeVisible();

        // Click edit button for the created user row
        const row = page.getByRole('row').filter({ hasText: 'Edit Me' });
        await row.getByTitle('Edit user').click();

        const dialog = page.getByRole('dialog');
        await expect(dialog).toBeVisible();
        await expect(dialog.getByRole('heading', { name: 'Edit user' })).toBeVisible();

        // Change last name
        await dialog.getByLabel('Last name').fill('Updated');
        await dialog.getByRole('button', { name: 'Save' }).click();

        await expect(dialog).not.toBeVisible();
        await expect(page.getByText('Edit Updated')).toBeVisible();

        // Cleanup
        await page.request.delete(`${API_URL}/users/${created.id}`, { headers });
    });

    test('delete user with confirm dialog', async ({ page }) => {
        // Create a user to delete
        const { headers } = await authState(page);
        const email = `deleteme-${Date.now()}@example.com`;
        const createRes = await page.request.post(`${API_URL}/users`, {
            headers,
            data: JSON.stringify({
                firstName: 'Delete',
                lastName: 'Me',
                email,
                password: 'Password123!',
                role: 'normal',
            }),
        });
        const created = await createRes.json();

        await page.goto('/admin/users');
        const row = page.getByRole('row').filter({ hasText: email });
        await expect(row).toBeVisible();

        await row.getByTitle('Delete user').click();

        // Confirm dialog
        const dialog = page.getByRole('dialog');
        await expect(dialog.getByText('Delete this user? This cannot be undone.')).toBeVisible();
        await dialog.getByRole('button', { name: 'Delete' }).click();

        // User should be gone from the table
        await expect(row).not.toBeVisible();
    });

    test('admin link visible in navbar dropdown for admin user', async ({ page }) => {
        await page.goto('/');
        await page
            .locator('header button')
            .filter({ has: page.locator('.rounded-full') })
            .click();
        await expect(page.getByRole('button', { name: 'Admin', exact: true })).toBeVisible();
    });

    test('admin link in navbar dropdown navigates to admin users page', async ({ page }) => {
        await page.goto('/');
        await page
            .locator('header button')
            .filter({ has: page.locator('.rounded-full') })
            .click();
        await page.getByRole('button', { name: 'Admin', exact: true }).click();
        await expect(page).toHaveURL('/admin/users');
    });
});
