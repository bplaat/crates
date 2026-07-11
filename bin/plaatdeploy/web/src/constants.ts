/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

export const STATUS_LABELS: Record<string, string> = {
    idle: 'Idle',
    building: 'Building',
    running: 'Running',
    failed: 'Failed',
};

export const DEPLOY_STATUS_LABELS: Record<string, string> = {
    pending: 'Pending',
    building: 'Building',
    succeeded: 'Succeeded',
    failed: 'Failed',
};
