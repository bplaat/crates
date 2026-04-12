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

async function createArchivedNote(page: Page, body: string): Promise<{ id: string }> {
    const headers = await authHeaders(page);
    // Create note then archive it via API
    const createRes = await page.request.post(`${API_URL}/notes`, {
        headers,
        data: new URLSearchParams({ body }).toString(),
    });
    const note = await createRes.json();
    await page.request.put(`${API_URL}/notes/${note.id}`, {
        headers,
        data: new URLSearchParams({ body, isPinned: 'false', isArchived: 'true', isTrashed: 'false' }).toString(),
    });
    return note;
}

async function noteOrder(page: Page, labels: string[]): Promise<string[]> {
    return page.locator('a[href^="/notes/"]').evaluateAll((els, expectedLabels) => {
        const order: string[] = [];
        for (const el of els) {
            const text = el.textContent ?? '';
            for (const label of expectedLabels) {
                if (text.includes(label)) {
                    order.push(label);
                    break;
                }
            }
        }
        return order;
    }, labels);
}

test.describe('Archive', () => {
    test('archive page loads with correct title', async ({ page }) => {
        await page.goto('/archive');
        await expect(page).toHaveTitle(/PlaatNotes.*Archive/);
    });

    test('shows empty state when no archived notes', async ({ page }) => {
        await page.goto('/archive');
        const emptyState = page.getByText('No archived notes.');
        const firstNote = page.locator('a[href^="/notes/"]').first();
        await expect(emptyState.or(firstNote)).toBeVisible();
    });

    test('archived note appears on archive page', async ({ page }) => {
        const note = await createArchivedNote(page, 'Archived note content test');

        await page.goto('/archive');
        await expect(page.getByText('Archived note content test')).toBeVisible();

        // Cleanup
        const headers = await authHeaders(page);
        await page.request.delete(`${API_URL}/notes/${note.id}`, { headers });
    });

    test('unarchive a note returns it to home', async ({ page }) => {
        const note = await createArchivedNote(page, 'Unarchive test note');

        await page.goto('/archive');
        await expect(page.getByText('Unarchive test note')).toBeVisible();

        // Click on the note to open it, then unarchive
        await page.getByText('Unarchive test note').click();
        await expect(page).toHaveURL(`/notes/${note.id}`);
        await page.getByTitle('Unarchive').click();
        await expect(page).toHaveURL('/');

        // Cleanup
        const headers = await authHeaders(page);
        await page.request.delete(`${API_URL}/notes/${note.id}`, { headers });
    });

    test('trash a note from archive page removes it from archive', async ({ page }) => {
        const note = await createArchivedNote(page, 'Trash from archive test');

        await page.goto('/archive');
        await page.getByText('Trash from archive test').click();
        await expect(page).toHaveURL(`/notes/${note.id}`);
        await page.getByTitle('Move to trash').click();
        await expect(page).toHaveURL('/trash');

        // Cleanup
        const headers = await authHeaders(page);
        await page.request.delete(`${API_URL}/notes/${note.id}`, { headers });
    });

    test('search filters archived notes', async ({ page }) => {
        const headers = await authHeaders(page);
        const note = await createArchivedNote(page, 'Archived searchable note xyz');

        await page.goto('/archive');
        const searchInput = page.getByPlaceholder('Search notes…');
        await searchInput.fill('__nonexistent_xyz_query__');
        await expect(page.getByText('No archived notes match your search.')).toBeVisible();

        await searchInput.clear();
        await page.request.delete(`${API_URL}/notes/${note.id}`, { headers });
    });

    test('dragging archived notes reorders them and persists after reload', async ({ page }) => {
        const labelA = `Archived reorder A ${Date.now()}`;
        const labelB = `Archived reorder B ${Date.now()}`;
        const noteA = await createArchivedNote(page, labelA);
        const noteB = await createArchivedNote(page, labelB);

        await page.goto('/archive');
        await expect.poll(() => noteOrder(page, [labelA, labelB])).toEqual([labelB, labelA]);

        const reorderPromise = page.waitForResponse(
            (res) => res.url().includes('/api/notes/archived/reorder') && res.request().method() === 'PUT',
        );
        await page
            .locator('div[draggable="true"]')
            .filter({ hasText: labelA })
            .dragTo(page.locator('div[draggable="true"]').filter({ hasText: labelB }));
        await reorderPromise;

        await expect.poll(() => noteOrder(page, [labelA, labelB])).toEqual([labelA, labelB]);

        await page.reload();
        await expect.poll(() => noteOrder(page, [labelA, labelB])).toEqual([labelA, labelB]);

        const headers = await authHeaders(page);
        await page.request.delete(`${API_URL}/notes/${noteA.id}`, { headers });
        await page.request.delete(`${API_URL}/notes/${noteB.id}`, { headers });
    });
});
