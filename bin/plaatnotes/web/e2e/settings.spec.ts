/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import { expect, test } from '@playwright/test';

test.describe('Settings', () => {
    test('account settings page loads with correct title', async ({ page }) => {
        await page.goto('/settings');
        await expect(page).toHaveTitle(/PlaatNotes.*Settings/);
        await expect(page.getByRole('heading', { name: 'Account' })).toBeVisible();
    });

    test('profile form is pre-filled with user data', async ({ page }) => {
        await page.goto('/settings');
        await expect(page.getByLabel('First name')).toHaveValue('Test');
        await expect(page.getByLabel('Last name')).toHaveValue('User');
        await expect(page.getByLabel('Email')).toHaveValue('test@example.com');
    });

    test('profile form shows both profile and password sections', async ({ page }) => {
        await page.goto('/settings');
        await expect(page.getByRole('heading', { name: 'Profile' })).toBeVisible();
        await expect(page.getByRole('heading', { name: 'Change password' })).toBeVisible();
    });

    test('save changes button is present', async ({ page }) => {
        await page.goto('/settings');
        await expect(page.getByRole('button', { name: 'Save changes' })).toBeVisible();
    });

    test('change password button is present', async ({ page }) => {
        await page.goto('/settings');
        await expect(page.getByRole('button', { name: 'Change password' })).toBeVisible();
    });

    test('sessions page loads with correct title', async ({ page }) => {
        await page.goto('/settings/sessions');
        await expect(page).toHaveTitle(/PlaatNotes.*Sessions/);
        await expect(page.getByRole('heading', { name: 'Active sessions' })).toBeVisible();
    });

    test('sessions page shows current session badge', async ({ page }) => {
        await page.goto('/settings/sessions');
        await expect(page.getByText('Current')).toBeVisible();
    });

    test('settings navigation sidebar has account, sessions, imports links', async ({ page }) => {
        await page.goto('/settings');
        await expect(page.getByRole('link', { name: 'Account' })).toBeVisible();
        await expect(page.getByRole('link', { name: 'Sessions' })).toBeVisible();
        await expect(page.getByRole('link', { name: 'Imports' })).toBeVisible();
    });

    test('imports page loads correctly', async ({ page }) => {
        await page.goto('/settings/imports');
        await expect(page).toHaveTitle(/PlaatNotes.*Imports/);
        await expect(page.getByRole('heading', { name: 'Google Keep' })).toBeVisible();
    });

    test('account page has theme and language selects', async ({ page }) => {
        await page.goto('/settings');
        await expect(page.getByLabel('Theme')).toBeVisible();
        await expect(page.getByLabel('Language')).toBeVisible();
    });

    test('save profile changes shows success message', async ({ page }) => {
        await page.goto('/settings');
        await page.getByRole('button', { name: 'Save changes' }).click();
        await expect(page.getByText('Changes saved!')).toBeVisible();
    });

    test('change password with wrong current password shows error', async ({ page }) => {
        await page.goto('/settings');
        // Use a too-short old password (< 8 chars) to trigger backend validation error
        await page.getByLabel('Current password').fill('short');
        await page.getByLabel('New password', { exact: true }).fill('newpassword123');
        await page.getByLabel('Confirm new password').fill('newpassword123');
        await page.getByRole('button', { name: 'Change password' }).click();
        await expect(page.getByText('Errors occurred')).toBeVisible();
    });

    test('revoke a non-current session with confirm dialog', async ({ page }) => {
        const API_URL = 'http://localhost:8080/api';

        // Create a second session via API login
        await page.request.post(`${API_URL}/auth/login`, {
            data: new URLSearchParams({ email: 'test@example.com', password: 'password' }).toString(),
            headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
        });

        await page.goto('/settings/sessions');

        // There should now be at least one non-current session with a Revoke button
        const revokeBtn = page.getByRole('button', { name: 'Revoke' }).first();
        await expect(revokeBtn).toBeVisible();
        await revokeBtn.click();

        await expect(page.getByText('Revoke this session? That device will be signed out.')).toBeVisible();
        await page.getByRole('button', { name: 'Revoke' }).last().click();

        await expect(page.getByText('Revoke this session? That device will be signed out.')).not.toBeVisible();
    });
});
