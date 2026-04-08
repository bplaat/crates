/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import { expect, test } from '@playwright/test';

test.describe('Auth', () => {
    test('shows login page when unauthenticated', async ({ page }) => {
        await page.goto('/');
        await expect(page).toHaveTitle(/PlaatNotes.*Login/);
        await expect(page.getByRole('heading', { name: 'PlaatNotes' })).toBeVisible();
        await expect(page.getByText('Sign in to your account')).toBeVisible();
        await expect(page.getByLabel('Email')).toBeVisible();
        await expect(page.getByLabel('Password')).toBeVisible();
        await expect(page.getByRole('button', { name: 'Sign in' })).toBeVisible();
    });

    test('shows error on invalid credentials', async ({ page }) => {
        await page.goto('/');
        await page.getByLabel('Email').fill('wrong@example.com');
        await page.getByLabel('Password').fill('wrongpassword');
        await page.getByRole('button', { name: 'Sign in' }).click();
        await expect(page.getByText('Invalid email or password.')).toBeVisible();
        await expect(page).toHaveURL('/');
    });

    test('redirects to home on successful login', async ({ page }) => {
        await page.goto('/');
        await page.getByLabel('Email').fill('test@example.com');
        await page.getByLabel('Password').fill('password');
        await page.getByRole('button', { name: 'Sign in' }).click();
        await expect(page).toHaveTitle('PlaatNotes');
    });

    test('logs out and returns to login page', async ({ page }) => {
        await page.goto('/');
        await page.getByLabel('Email').fill('test@example.com');
        await page.getByLabel('Password').fill('password');
        await page.getByRole('button', { name: 'Sign in' }).click();
        await expect(page).toHaveTitle('PlaatNotes');

        // Open user dropdown and click Logout
        await page
            .locator('header button')
            .filter({ has: page.locator('.rounded-full') })
            .click();
        await page.getByRole('button', { name: 'Logout' }).click();

        // Should return to login page
        await expect(page.getByRole('button', { name: 'Sign in' })).toBeVisible();
    });

    test('unauthenticated access to protected route shows login page', async ({ page }) => {
        await page.goto('/archive');
        await expect(page.getByRole('button', { name: 'Sign in' })).toBeVisible();
        await expect(page.getByText('Sign in to your account')).toBeVisible();
    });
});
