/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import { type Page, expect, test } from '@playwright/test';

const API_URL = 'http://localhost:8080/api';

async function authHeaders(page: Page): Promise<Record<string, string>> {
    if (!page.url().startsWith('http')) await page.goto('/');
    const token = await page.evaluate(() => localStorage.getItem('token') ?? '');
    return {
        Authorization: `Bearer ${token}`,
        'Content-Type': 'application/x-www-form-urlencoded',
    };
}

async function createTrashedNote(page: Page, body: string): Promise<{ id: string }> {
    const headers = await authHeaders(page);
    const createRes = await page.request.post(`${API_URL}/notes`, {
        headers,
        data: new URLSearchParams({ body }).toString(),
    });
    const note = await createRes.json();
    await page.request.put(`${API_URL}/notes/${note.id}`, {
        headers,
        data: new URLSearchParams({ body, isPinned: 'false', isArchived: 'false', isTrashed: 'true' }).toString(),
    });
    return note;
}

test.describe('Trash', () => {
    test('trash page loads with correct title', async ({ page }) => {
        await page.goto('/trash');
        await expect(page).toHaveTitle(/PlaatNotes.*Trash/);
    });

    test('shows empty state when no trashed notes', async ({ page }) => {
        await page.goto('/trash');
        const emptyState = page.getByText('Trash is empty.');
        const firstNote = page.locator('a[href^="/notes/"]').first();
        await expect(emptyState.or(firstNote)).toBeVisible();
    });

    test('trashed note appears on trash page', async ({ page }) => {
        const note = await createTrashedNote(page, 'Trashed note content test');

        await page.goto('/trash');
        await expect(page.getByText('Trashed note content test')).toBeVisible();

        // Cleanup
        const headers = await authHeaders(page);
        await page.request.delete(`${API_URL}/notes/${note.id}`, { headers });
    });

    test('restore a note from trash returns it to home', async ({ page }) => {
        const note = await createTrashedNote(page, 'Restore test note');

        await page.goto('/trash');

        // Click restore button on the note card
        await page.locator('a').filter({ hasText: 'Restore test note' }).getByTitle('Restore').click();

        // Note should disappear from trash
        await expect(page.getByText('Restore test note')).not.toBeVisible();

        // Cleanup
        const headers = await authHeaders(page);
        await page.request.delete(`${API_URL}/notes/${note.id}`, { headers });
    });

    test('delete forever shows confirm dialog and removes note', async ({ page }) => {
        const note = await createTrashedNote(page, 'Delete forever test note');

        await page.goto('/trash');
        await expect(page.getByText('Delete forever test note')).toBeVisible();

        // Click delete forever button
        await page.locator('a').filter({ hasText: 'Delete forever test note' }).getByTitle('Delete forever').click();

        // Confirm dialog should appear
        await expect(page.getByText('Delete this note forever? This cannot be undone.')).toBeVisible();
        await page.getByRole('button', { name: 'Delete forever' }).last().click();

        // Note should be gone
        await expect(page.getByText('Delete forever test note')).not.toBeVisible();
    });

    test('empty trash shows confirm dialog and clears all notes', async ({ page }) => {
        const note1 = await createTrashedNote(page, 'Empty trash test note 1');
        const note2 = await createTrashedNote(page, 'Empty trash test note 2');

        await page.goto('/trash');
        await expect(page.getByText('Empty trash test note 1')).toBeVisible();
        await expect(page.getByText('Empty trash test note 2')).toBeVisible();

        await page.getByRole('button', { name: 'Empty trash' }).click();

        // Confirm dialog
        await expect(page.getByText('Empty trash? All trashed notes will be deleted forever.')).toBeVisible();
        await page.getByRole('button', { name: 'Empty trash' }).last().click();

        // Both notes gone
        await expect(page.getByText('Empty trash test note 1')).not.toBeVisible();
        await expect(page.getByText('Empty trash test note 2')).not.toBeVisible();
        await expect(page.getByText('Trash is empty.')).toBeVisible();
    });

    test('trash page shows hint text when notes are present', async ({ page }) => {
        const note = await createTrashedNote(page, 'Hint text test note');

        await page.goto('/trash');
        await expect(page.getByText('Notes in trash will be permanently deleted after some time.')).toBeVisible();

        // Cleanup
        const headers = await authHeaders(page);
        await page.request.delete(`${API_URL}/notes/${note.id}`, { headers });
    });

    test('search filters trashed notes', async ({ page }) => {
        const note = await createTrashedNote(page, 'Searchable trashed note xyz');

        await page.goto('/trash');
        const searchInput = page.getByPlaceholder('Search notes…');
        await searchInput.fill('__nonexistent_xyz_query__');
        await expect(page.getByText('No trashed notes match your search.')).toBeVisible();

        await searchInput.clear();

        const headers = await authHeaders(page);
        await page.request.delete(`${API_URL}/notes/${note.id}`, { headers });
    });
});
