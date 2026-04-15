/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import { Buffer } from 'node:buffer';
import { expect, test, type Page } from '@playwright/test';

const API_URL = `http://localhost:${process.env.PLAYWRIGHT_PORT ?? '8080'}/api`;
const GOOGLE_KEEP_ZIP_B64 =
    'UEsDBBQAAAAAAHNrjFwIITQ31QAAANUAAAAXAAAAVGFrZW91dC9LZWVwL25vdGUwLmpzb257InRpdGxlIjogIkltcG9ydGVkIG5vdGUgb25lIiwgInRleHRDb250ZW50IjogIkltcG9ydGVkIGJvZHkgb25lIiwgImlzUGlubmVkIjogZmFsc2UsICJpc0FyY2hpdmVkIjogZmFsc2UsICJpc1RyYXNoZWQiOiBmYWxzZSwgImNyZWF0ZWRUaW1lc3RhbXBVc2VjIjogMTcwMDAwMDAwMDAwMDAwMCwgInVzZXJFZGl0ZWRUaW1lc3RhbXBVc2VjIjogMTcwMDAwMDAwMDAwMDAwMH1QSwMEFAAAAAAAc2uMXEHt81vVAAAA1QAAABcAAABUYWtlb3V0L0tlZXAvbm90ZTEuanNvbnsidGl0bGUiOiAiSW1wb3J0ZWQgbm90ZSB0d28iLCAidGV4dENvbnRlbnQiOiAiSW1wb3J0ZWQgYm9keSB0d28iLCAiaXNQaW5uZWQiOiBmYWxzZSwgImlzQXJjaGl2ZWQiOiBmYWxzZSwgImlzVHJhc2hlZCI6IGZhbHNlLCAiY3JlYXRlZFRpbWVzdGFtcFVzZWMiOiAxNzAwMDAwMDAwMDAwMDAwLCAidXNlckVkaXRlZFRpbWVzdGFtcFVzZWMiOiAxNzAwMDAwMDAwMDAwMDAwfVBLAQIUAxQAAAAAAHNrjFwIITQ31QAAANUAAAAXAAAAAAAAAAAAAACAAQAAAABUYWtlb3V0L0tlZXAvbm90ZTAuanNvblBLAQIUAxQAAAAAAHNrjFxB7fNb1QAAANUAAAAXAAAAAAAAAAAAAACAAQoBAABUYWtlb3V0L0tlZXAvbm90ZTEuanNvblBLBQYAAAAAAgACAIoAAAAUAgAAAAA=';

async function authState(page: Page): Promise<{ token: string; userId: string; headers: Record<string, string> }> {
    if (!page.url().startsWith('http')) await page.goto('/');
    const token = await page.evaluate(() => localStorage.getItem('token') ?? '');
    const headers = { Authorization: `Bearer ${token}`, 'Content-Type': 'application/json' };
    const res = await page.request.get(`${API_URL}/auth/validate`, { headers });
    const { user } = await res.json();
    return { token, userId: user.id, headers };
}

async function deleteNotesByQuery(page: Page, query: string): Promise<void> {
    const { userId, headers } = await authState(page);
    const res = await page.request.get(`${API_URL}/users/${userId}/notes?page=1&q=${encodeURIComponent(query)}`, {
        headers,
    });
    const data: { data: Array<{ id: string }> } = await res.json();
    for (const note of data.data) {
        await page.request.delete(`${API_URL}/notes/${note.id}`, { headers });
    }
}

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
        await expect(page.getByText('Errors occurred.')).toBeVisible();
    });

    test('revoke a non-current session with confirm dialog', async ({ page }) => {
        // Create a second session via API login
        await page.request.post(`${API_URL}/auth/login`, {
            data: JSON.stringify({ email: 'test@example.com', password: 'password' }),
            headers: { 'Content-Type': 'application/json' },
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

    test('imports notes from a Google Keep zip upload', async ({ page }) => {
        await page.goto('/settings/imports');
        await page.getByLabel('Takeout zip file').setInputFiles({
            name: 'takeout.zip',
            mimeType: 'application/zip',
            buffer: Buffer.from(GOOGLE_KEEP_ZIP_B64, 'base64'),
        });
        await page.getByRole('button', { name: 'Import' }).click();

        await expect(page.getByText('Imported 2 notes!')).toBeVisible();

        await page.goto('/');
        await expect(page.locator('a[href^="/notes/"]').filter({ hasText: 'Imported note one' }).first()).toBeVisible();
        await expect(page.locator('a[href^="/notes/"]').filter({ hasText: 'Imported note two' }).first()).toBeVisible();

        await deleteNotesByQuery(page, 'Imported note');
    });
});
