/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import { expect, test } from '@playwright/test';

test.describe('Home', () => {
    test('loads home page with correct title', async ({ page }) => {
        await page.goto('/');
        await expect(page).toHaveTitle('PlaatNotes');
    });

    test('shows empty state or notes grid', async ({ page }) => {
        await page.goto('/');
        const emptyState = page.getByText('No notes yet. Create one!');
        const firstNote = page.locator('a[href^="/notes/"]').first();
        await expect(emptyState.or(firstNote)).toBeVisible();
    });

    test('FAB button navigates to create note page', async ({ page }) => {
        await page.goto('/');
        await page.getByTitle('Create note').click();
        await expect(page).toHaveURL('/notes/create');
        await expect(page).toHaveTitle(/PlaatNotes.*Create Note/);
    });

    test('search bar is visible and filters notes', async ({ page }) => {
        await page.goto('/');
        const searchInput = page.getByPlaceholder('Search notes…');
        await expect(searchInput).toBeVisible();

        await searchInput.fill('__nonexistent_query_xyz__');
        await expect(page.getByText('No notes match your search.')).toBeVisible();

        await searchInput.clear();
    });

    test('navbar shows user avatar and name', async ({ page }) => {
        await page.goto('/');
        // Avatar initials: TU for Test User
        await expect(page.locator('header').getByText('TU')).toBeVisible();
        await expect(page.locator('header').getByText('Test User')).toBeVisible();
    });

    test('user dropdown menu contains Settings and Sign out', async ({ page }) => {
        await page.goto('/');
        await page
            .locator('header button')
            .filter({ has: page.locator('.rounded-full') })
            .click();
        await expect(page.getByRole('button', { name: 'Settings' })).toBeVisible();
        await expect(page.getByRole('button', { name: 'Sign out' })).toBeVisible();
    });

    test('sidebar links navigate correctly', async ({ page }) => {
        await page.goto('/');
        await page.getByRole('link', { name: 'Archive', exact: true }).click();
        await expect(page).toHaveURL('/archive');

        await page.getByRole('link', { name: 'Trash', exact: true }).click();
        await expect(page).toHaveURL('/trash');

        await page.getByRole('link', { name: 'Notes', exact: true }).click();
        await expect(page).toHaveURL('/');
    });

    test('unknown route shows not-found page', async ({ page }) => {
        await page.goto('/this-route-does-not-exist');
        await expect(page.getByRole('heading', { name: '404' })).toBeVisible();
        await expect(page.getByText('No idea how you got here')).toBeVisible();
        await expect(page.getByRole('button', { name: 'Go home' })).toBeVisible();
    });
});

test.describe('Home - Note card actions', () => {
    const API_URL = 'http://localhost:8080/api';

    async function authHeaders(page: import('@playwright/test').Page): Promise<Record<string, string>> {
        if (!page.url().startsWith('http')) await page.goto('/');
        const token = await page.evaluate(() => localStorage.getItem('token') ?? '');
        return { Authorization: `Bearer ${token}`, 'Content-Type': 'application/x-www-form-urlencoded' };
    }

    async function createNote(
        page: import('@playwright/test').Page,
        fields: Record<string, string>,
    ): Promise<{ id: string }> {
        const headers = await authHeaders(page);
        const res = await page.request.post(`${API_URL}/notes`, {
            headers,
            data: new URLSearchParams(fields).toString(),
        });
        return res.json();
    }

    async function deleteNote(page: import('@playwright/test').Page, id: string): Promise<void> {
        const headers = await authHeaders(page);
        await page.request.delete(`${API_URL}/notes/${id}`, { headers });
    }

    test('pin a note via note card button on home page', async ({ page }) => {
        const note = await createNote(page, { body: 'Card pin test note', title: 'Card Pin Note' });

        await page.goto('/');
        const card = page.locator('a[href^="/notes/"]').filter({ hasText: 'Card Pin Note' });
        await card.hover();
        await card.getByTitle('Pin').click();

        // Pinned section should now appear
        await expect(page.getByText('Pinned')).toBeVisible();

        await deleteNote(page, note.id);
    });

    test('archive a note via note card button on home page', async ({ page }) => {
        const note = await createNote(page, { body: 'Card archive test note', title: 'Card Archive Note' });

        await page.goto('/');
        const card = page.locator('a[href^="/notes/"]').filter({ hasText: 'Card Archive Note' });
        await card.hover();
        await card.getByTitle('Archive').click();

        await expect(page.locator('a[href^="/notes/"]').filter({ hasText: 'Card Archive Note' })).not.toBeVisible();

        await deleteNote(page, note.id);
    });

    test('trash a note via note card button on home page', async ({ page }) => {
        const note = await createNote(page, { body: 'Card trash test note', title: 'Card Trash Note' });

        await page.goto('/');
        const card = page.locator('a[href^="/notes/"]').filter({ hasText: 'Card Trash Note' });
        await card.hover();
        await card.getByTitle('Move to trash').click();

        await expect(page.locator('a[href^="/notes/"]').filter({ hasText: 'Card Trash Note' })).not.toBeVisible();

        await deleteNote(page, note.id);
    });

    test('"Pinned" section heading visible when pinned notes exist', async ({ page }) => {
        const headers = await authHeaders(page);
        const note = await createNote(page, { body: 'Pinned section test' });
        await page.request.put(`${API_URL}/notes/${note.id}`, {
            headers,
            data: new URLSearchParams({
                body: 'Pinned section test',
                isPinned: 'true',
                isArchived: 'false',
                isTrashed: 'false',
            }).toString(),
        });

        await page.goto('/');
        await expect(page.getByRole('heading', { name: 'Pinned' })).toBeVisible();

        await deleteNote(page, note.id);
    });
});
