/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 * SPDX-License-Identifier: MIT
 */

import { signal } from '@preact/signals';

const CURRENT_TEAM_KEY = 'current-team-id';

export const $currentTeamId = signal<string | null>(localStorage.getItem(CURRENT_TEAM_KEY));

export function setCurrentTeamId(teamId: string | null) {
    $currentTeamId.value = teamId;
    if (teamId) localStorage.setItem(CURRENT_TEAM_KEY, teamId);
    else localStorage.removeItem(CURRENT_TEAM_KEY);
}
