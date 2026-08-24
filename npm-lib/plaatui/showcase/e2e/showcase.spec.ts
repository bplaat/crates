/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import { expect, test } from '@playwright/test';

test.describe('PlaatUI showcase', () => {
    test('renders the component catalog', async ({ page }) => {
        await page.goto('./');

        await expect(page).toHaveTitle('PlaatUI Showcase');
        await expect(page.getByRole('heading', { name: 'PlaatUI component showcase' })).toBeVisible();
        await expect(page.locator('.showcase-section')).toHaveCount(15);
        await expect(page.locator('.sidebar-version')).toHaveText('v0.1.0');
        await expect(page.getByRole('heading', { name: 'Buttons' })).toBeVisible();
        await expect(page.getByRole('heading', { name: 'Icons' })).toBeVisible();
    });

    test('filters and clears the component catalog', async ({ page }) => {
        await page.goto('./');
        const search = page.locator('.layout > .navbar input[type="search"]');

        await search.fill('dialogs');
        await expect(page.locator('.showcase-section')).toHaveCount(1);
        await expect(page.getByRole('heading', { name: 'Dialogs' })).toBeVisible();

        await search.fill('missing component');
        await expect(page.getByText('No components match "missing component"')).toBeVisible();

        await page.locator('.navbar > .navbar-container > .navbar-search .search-clear').click();
        await expect(search).toHaveValue('');
        await expect(page.locator('.showcase-section')).toHaveCount(15);
    });

    test('persists the selected theme', async ({ page }) => {
        await page.goto('./');
        await page.evaluate(() => localStorage.setItem('plaatui-showcase-theme', 'light'));
        await page.reload();

        await expect(page.locator('html')).not.toHaveClass(/dark/);
        await page.getByTitle('Toggle theme').click();
        await expect(page.locator('html')).toHaveClass(/dark/);
        expect(await page.evaluate(() => localStorage.getItem('plaatui-showcase-theme'))).toBe('dark');

        await page.reload();
        await expect(page.locator('html')).toHaveClass(/dark/);
    });

    test('updates input, select and search controls', async ({ page }) => {
        await page.goto('./');
        const section = page.locator('#inputs');
        const textInput = section.locator('input[type="text"]');

        await textInput.fill('Hello PlaatUI');
        await expect(textInput).toHaveValue('Hello PlaatUI');

        await section.locator('select').selectOption('cog');
        await expect(section.locator('select')).toHaveValue('cog');

        const search = section.locator('input[type="search"]');
        await search.fill('notes');
        await section.locator('.search-clear').click();
        await expect(search).toHaveValue('');
    });

    test('opens and closes basic and form dialogs', async ({ page }) => {
        await page.goto('./');
        const section = page.locator('#dialogs');

        await section.getByRole('button', { name: 'Open dialog' }).click();
        const basicDialog = page.getByRole('dialog');
        await expect(basicDialog.getByRole('heading', { name: 'Example dialog' })).toBeVisible();
        await basicDialog.getByRole('button', { name: 'Got it' }).click();
        await expect(basicDialog).not.toBeVisible();

        await section.getByRole('button', { name: 'Open form dialog' }).click();
        const formDialog = page.getByRole('dialog');
        await formDialog.getByLabel('Title').fill('E2E note');
        await formDialog.getByLabel('Tag').selectOption('Work');
        await formDialog.getByRole('button', { name: 'Save' }).click();
        await expect(formDialog).not.toBeVisible();
    });

    test('gates destructive confirmation on the required text', async ({ page }) => {
        await page.goto('./');
        await page.locator('#dialogs').getByRole('button', { name: 'Delete item' }).click();

        const dialog = page.getByRole('dialog');
        const deleteButton = dialog.getByRole('button', { name: 'Delete' });
        await expect(dialog.getByText('This action cannot be undone.')).toBeVisible();
        await expect(deleteButton).toBeDisabled();

        await dialog.getByLabel('Type "my-item" to confirm').fill('wrong');
        await expect(deleteButton).toBeDisabled();
        await dialog.getByLabel('Type "my-item" to confirm').fill('my-item');
        await expect(deleteButton).toBeEnabled();
        await deleteButton.click();
        await expect(dialog).not.toBeVisible();
    });

    test('handles dropdown and sidebar interactions', async ({ page }) => {
        await page.goto('./');

        const navbar = page.locator('.layout > .navbar');
        await navbar.locator('.navbar-user').click();
        await expect(navbar.getByRole('button', { name: 'Profile' })).toBeVisible();
        await page.getByRole('heading', { name: 'PlaatUI component showcase' }).click();
        await expect(navbar.getByRole('button', { name: 'Profile' })).not.toBeVisible();

        const sidebarDemo = page.locator('.showcase-sidebar-demo');
        await sidebarDemo.getByTitle('Settings').click();
        await expect(sidebarDemo.getByText('Selected').locator('..').getByText('Settings')).toBeVisible();
        await expect(sidebarDemo.getByTitle('Settings')).toHaveClass(/is-active/);
    });
});
