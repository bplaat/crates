/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import { type Page, expect, test } from '@playwright/test';

const API_URL = `http://localhost:${process.env.PLAYWRIGHT_PORT ?? '8080'}/api`;

async function authHeaders(page: Page): Promise<Record<string, string>> {
    if (!page.url().startsWith('http')) await page.goto('/');
    const token = await page.evaluate(() => localStorage.getItem('token') ?? '');
    return {
        Authorization: `Bearer ${token}`,
        'Content-Type': 'application/x-www-form-urlencoded',
    };
}

async function createNote(page: Page, fields: Record<string, string>): Promise<{ id: string }> {
    const headers = await authHeaders(page);
    const res = await page.request.post(`${API_URL}/notes`, {
        headers,
        data: new URLSearchParams(fields).toString(),
    });
    return res.json();
}

async function deleteNote(page: Page, id: string): Promise<void> {
    const headers = await authHeaders(page);
    await page.request.delete(`${API_URL}/notes/${id}`, { headers });
}

test.describe('Notes', () => {
    test('create note with title and body', async ({ page }) => {
        await page.goto('/notes/create');
        await expect(page).toHaveTitle(/Create Note/);

        await page.getByPlaceholder('Title').fill('My Test Note');
        await page.locator('[contenteditable]').fill('Hello world content');
        await page.waitForURL((url) => url.pathname.startsWith('/notes/') && url.pathname !== '/notes/create');

        await expect(page).toHaveURL(/\/notes\//);
        await expect(page).toHaveTitle(/My Test Note/);

        // Cleanup
        const noteId = page.url().split('/notes/')[1];
        await deleteNote(page, noteId);
    });

    test('back button from create page returns to home', async ({ page }) => {
        await page.goto('/notes/create');
        await page.getByTitle('Back').click();
        await expect(page).toHaveURL('/');
    });

    test('leaving create page without a body returns to home without creating a note', async ({ page }) => {
        await page.goto('/notes/create');
        await page.getByPlaceholder('Title').fill('Draft without body');
        await page.waitForTimeout(700);
        await expect(page).toHaveURL('/notes/create');
        await page.getByTitle('Back').click();
        await expect(page).toHaveURL('/');
    });

    test('view and edit a note with auto-save', async ({ page }) => {
        const note = await createNote(page, { body: 'Original content', title: 'Edit Test Note' });

        await page.goto(`/notes/${note.id}`);
        await expect(page).toHaveTitle(/Edit Test Note/);

        // Edit the body and wait for auto-save PUT
        const savePromise = page.waitForResponse(
            (res) => res.url().includes(`/notes/${note.id}`) && res.request().method() === 'PUT',
        );
        await page.locator('[contenteditable]').fill('Updated content');
        await savePromise;

        await expect(page.getByText('Saved!')).toBeVisible();

        await deleteNote(page, note.id);
    });

    test('pin and unpin a note', async ({ page }) => {
        const note = await createNote(page, { body: 'Pin test note' });

        await page.goto(`/notes/${note.id}`);

        // Pin the note
        await page.getByTitle('Pin').click();
        await expect(page.getByTitle('Unpin')).toBeVisible();

        // Unpin the note
        await page.getByTitle('Unpin').click();
        await expect(page.getByTitle('Pin')).toBeVisible();

        await deleteNote(page, note.id);
    });

    test('archive a note from note page navigates to archive', async ({ page }) => {
        const note = await createNote(page, { body: 'Archive from note test' });

        await page.goto(`/notes/${note.id}`);
        await page.getByTitle('Archive').click();
        await expect(page).toHaveURL('/archive');

        await deleteNote(page, note.id);
    });

    test('trash a note from note page navigates to trash', async ({ page }) => {
        const note = await createNote(page, { body: 'Trash from note test' });

        await page.goto(`/notes/${note.id}`);
        await page.getByTitle('Move to trash').click();
        await expect(page).toHaveURL('/trash');

        await deleteNote(page, note.id);
    });

    test('pin toggle on create note page', async ({ page }) => {
        await page.goto('/notes/create');
        await expect(page.getByTitle('Pin')).toBeVisible();
        await page.getByTitle('Pin').click();
        await expect(page.getByTitle('Unpin')).toBeVisible();
        await page.getByTitle('Unpin').click();
        await expect(page.getByTitle('Pin')).toBeVisible();
    });

    test('rich editor is focused on create page load', async ({ page }) => {
        await page.goto('/notes/create');
        await expect(page.locator('[contenteditable]')).toBeFocused();
    });

    test('note is not created until the body has content', async ({ page }) => {
        await page.goto('/notes/create');
        await page.getByPlaceholder('Title').fill('Only title no body');
        await page.waitForTimeout(700);
        await expect(page).toHaveURL('/notes/create');

        await page.locator('[contenteditable]').fill('Now the note can be created');
        await page.waitForURL((url) => url.pathname.startsWith('/notes/') && url.pathname !== '/notes/create');

        const noteId = page.url().split('/notes/')[1];
        await deleteNote(page, noteId);
    });

    test('back button on archived note navigates to archive', async ({ page }) => {
        const note = await createNote(page, { body: 'Back from archive note' });
        const headers = await authHeaders(page);
        await page.request.put(`${API_URL}/notes/${note.id}`, {
            headers,
            data: new URLSearchParams({
                body: 'Back from archive note',
                isPinned: 'false',
                isArchived: 'true',
                isTrashed: 'false',
            }).toString(),
        });

        await page.goto(`/notes/${note.id}`);
        await page.getByTitle('Back').click();
        await expect(page).toHaveURL('/archive');

        await deleteNote(page, note.id);
    });

    test('back button on trashed note navigates to trash', async ({ page }) => {
        const note = await createNote(page, { body: 'Back from trash note' });
        const headers = await authHeaders(page);
        await page.request.put(`${API_URL}/notes/${note.id}`, {
            headers,
            data: new URLSearchParams({
                body: 'Back from trash note',
                isPinned: 'false',
                isArchived: 'false',
                isTrashed: 'true',
            }).toString(),
        });

        await page.goto(`/notes/${note.id}`);
        await page.getByTitle('Back').click();
        await expect(page).toHaveURL('/trash');

        await deleteNote(page, note.id);
    });

    test('title editing triggers auto-save', async ({ page }) => {
        const note = await createNote(page, { body: 'Title auto-save test' });

        await page.goto(`/notes/${note.id}`);

        const savePromise = page.waitForResponse(
            (res) => res.url().includes(`/notes/${note.id}`) && res.request().method() === 'PUT',
        );
        await page.getByPlaceholder('Title').fill('Auto-saved title');
        await savePromise;

        await expect(page.getByText('Saved!')).toBeVisible();

        await deleteNote(page, note.id);
    });

    test('"Saving…" status shows while debounce is pending', async ({ page }) => {
        const note = await createNote(page, { body: 'Saving status test' });

        await page.goto(`/notes/${note.id}`);
        await page.locator('[contenteditable]').fill('Trigger saving status');
        await expect(page.getByText('Saving…')).toBeVisible();

        await deleteNote(page, note.id);
    });
});
