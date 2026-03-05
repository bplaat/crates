/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import { test as setup } from '@playwright/test';
import path from 'node:path';

const authFile = path.join(import.meta.dirname, '../playwright/.auth/admin.json');

setup('authenticate as admin user', async ({ page }) => {
    await page.goto('/');
    await page.getByLabel(/email/i).fill('testadmin@example.com');
    await page.getByLabel(/password/i).fill('password');
    await page.getByRole('button', { name: 'Sign in' }).click();
    await page.waitForFunction(() => localStorage.getItem('token') !== null);
    await page.context().storageState({ path: authFile });
});
