/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import { defineConfig, devices } from '@playwright/test';

export default defineConfig({
    testDir: './e2e',
    fullyParallel: true,
    forbidOnly: !!process.env.CI,
    retries: 2,
    workers: process.env.CI ? 1 : undefined,
    reporter: process.env.CI ? [['list'], ['junit', { outputFile: 'test-results/junit.xml' }]] : 'list',
    expect: {
        timeout: 10_000,
    },
    use: {
        baseURL: 'http://localhost:8080',
        trace: 'on-first-retry',
    },
    projects: [
        // Setup projects - run before test projects that depend on them
        {
            name: 'setup-normal',
            testMatch: /auth-normal\.setup\.ts/,
        },
        {
            name: 'setup-admin',
            testMatch: /auth-admin\.setup\.ts/,
        },

        // Unauthenticated tests (auth flows)
        {
            name: 'auth',
            testMatch: ['**/auth.spec.ts'],
            use: { ...devices['Desktop Chrome'] },
        },

        // Normal user tests
        {
            name: 'normal',
            testMatch: [
                '**/home.spec.ts',
                '**/notes.spec.ts',
                '**/archive.spec.ts',
                '**/trash.spec.ts',
                '**/settings.spec.ts',
            ],
            use: {
                ...devices['Desktop Chrome'],
                storageState: 'playwright/.auth/normal.json',
            },
            dependencies: ['setup-normal'],
        },

        // Admin user tests
        {
            name: 'admin',
            testMatch: ['**/admin.spec.ts'],
            use: {
                ...devices['Desktop Chrome'],
                storageState: 'playwright/.auth/admin.json',
            },
            dependencies: ['setup-admin'],
        },
    ],
    webServer: [
        {
            command: 'cargo run -- serve-e2e',
            cwd: '..',
            url: 'http://localhost:8080/',
            reuseExistingServer: !process.env.CI,
        },
    ],
});
